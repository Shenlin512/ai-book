use crate::models::*;
use crate::retrieval::{clip, search};

pub fn build_context(
    bundle: &ProjectBundle,
    settings: &AppSettings,
    req: &GenerateRequest,
) -> ContextPreview {
    let target = target_of(req);
    let chapter = req
        .chapter_id
        .as_ref()
        .and_then(|id| bundle.chapters.iter().find(|c| c.id == *id));
    let outline = req
        .outline_id
        .as_ref()
        .and_then(|id| bundle.outlines.iter().find(|o| o.id == *id))
        .or_else(|| {
            chapter
                .and_then(|c| c.outline_id.as_ref())
                .and_then(|id| bundle.outlines.iter().find(|o| o.id == *id))
        });
    let character = req
        .character_id
        .as_ref()
        .and_then(|id| bundle.characters.iter().find(|c| c.id == *id));
    let entry = req
        .entry_id
        .as_ref()
        .and_then(|id| bundle.entries.iter().find(|e| e.id == *id));

    let query = compose_query(req, chapter, outline, character, entry);
    let retrieved = if settings.include_retrieval && req.mode != "memory" {
        search(bundle, &query, settings.retrieve_k.max(1) as usize)
    } else {
        Vec::new()
    };

    let mut items: Vec<ContextItem> = Vec::new();
    let budget = settings.context_budget.max(1200) as usize;

    if settings.include_outline {
        if let Some(o) = outline {
            push_unique(
                &mut items,
                ContextItem {
                    kind: "outline".into(),
                    id: o.id.clone(),
                    title: format!("当前大纲 · {}", o.title),
                    snippet: clip(&o.content, 500),
                    score: 99.0,
                },
            );
        }
    }

    if settings.include_characters {
        for id in &req.character_ids {
            if let Some(c) = bundle.characters.iter().find(|c| c.id == *id) {
                push_unique(&mut items, character_item(c, 98.0));
            }
        }
        if let Some(c) = character {
            push_unique(&mut items, character_item(c, 99.0));
        }
        for c in mentioned_characters(bundle, &query) {
            push_unique(&mut items, character_item(c, 90.0));
        }
    }

    if settings.include_entries {
        for id in &req.entry_ids {
            if let Some(e) = bundle.entries.iter().find(|e| e.id == *id) {
                push_unique(&mut items, entry_item(e, 97.0));
            }
        }
        if let Some(e) = entry {
            push_unique(&mut items, entry_item(e, 99.0));
        }
    }

    if settings.include_memories {
        for m in bundle.memories.iter().take(16) {
            push_unique(
                &mut items,
                ContextItem {
                    kind: "memory".into(),
                    id: m.id.clone(),
                    title: format!("记忆 · {}", m.title),
                    snippet: clip(&m.content, 260),
                    score: 80.0,
                },
            );
        }
    }

    if req.mode == "memory" {
        for (idx, ch) in historical_chapters(bundle, None).into_iter().enumerate() {
            push_unique(
                &mut items,
                ContextItem {
                    kind: "chapter".into(),
                    id: ch.id.clone(),
                    title: format!("历史章节 · {}", ch.title),
                    snippet: chapter_extract(ch, if idx < 4 { 900 } else { 280 }),
                    score: 96.0 - idx as f32,
                },
            );
        }
    } else if let Some(ch) = chapter {
        let prev = previous_summaries(bundle, ch);
        if !prev.is_empty() {
            push_unique(
                &mut items,
                ContextItem {
                    kind: "summary".into(),
                    id: format!("prev-{}", ch.id),
                    title: "前文摘要".into(),
                    snippet: prev,
                    score: 95.0,
                },
            );
        }
    }

    for item in retrieved {
        if chapter.is_some() && item.kind == "chapter" && Some(&item.id) == req.chapter_id.as_ref() {
            continue;
        }
        push_unique(&mut items, item);
    }

    items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    trim_budget(&mut items, budget);

    ContextPreview {
        items: items.clone(),
        system_prompt: build_system(bundle, settings, req, &target),
        user_prompt: build_user(bundle, req, &target, chapter, outline, character, entry, &items),
    }
}

fn target_of(req: &GenerateRequest) -> String {
    if !req.target.trim().is_empty() {
        return req.target.trim().to_string();
    }
    if req.mode == "memory" {
        return "memories".into();
    }
    if req.character_id.is_some() {
        return "characters".into();
    }
    if req.entry_id.is_some() {
        return "entries".into();
    }
    if req.outline_id.is_some() && req.chapter_id.is_none() {
        return "outlines".into();
    }
    "chapters".into()
}

fn compose_query(
    req: &GenerateRequest,
    chapter: Option<&Chapter>,
    outline: Option<&Outline>,
    character: Option<&Character>,
    entry: Option<&Entry>,
) -> String {
    let mut parts = vec![req.instruction.clone()];
    if let Some(snap) = &req.object_snapshot {
        parts.push(snap.clone());
    }
    if let Some(sel) = &req.selected_text {
        parts.push(sel.clone());
    }
    if let Some(ch) = chapter {
        parts.push(ch.title.clone());
        parts.push(ch.summary.clone());
        parts.push(tail(&ch.content, 800));
    }
    if let Some(o) = outline {
        parts.push(o.title.clone());
        parts.push(o.content.clone());
    }
    if let Some(c) = character {
        parts.push(c.name.clone());
        parts.push(c.background.clone());
        parts.push(c.current_state.clone());
    }
    if let Some(e) = entry {
        parts.push(e.title.clone());
        parts.push(e.content.clone());
    }
    parts.join("\n")
}

fn mentioned_characters<'a>(bundle: &'a ProjectBundle, query: &str) -> Vec<&'a Character> {
    bundle
        .characters
        .iter()
        .filter(|c| {
            if query.contains(&c.name) {
                return true;
            }
            c.aliases
                .split(|ch| matches!(ch, ',' | '，' | ';' | '；' | '、' | ' '))
                .any(|alias| {
                    let alias = alias.trim();
                    !alias.is_empty() && query.contains(alias)
                })
        })
        .collect()
}

fn character_item(c: &Character, score: f32) -> ContextItem {
    ContextItem {
        kind: "character".into(),
        id: c.id.clone(),
        title: format!("角色 · {}", c.name),
        snippet: clip(&format_character(c), 420),
        score,
    }
}

fn format_character(c: &Character) -> String {
    format!(
        "姓名：{}\n别称：{}\n定位：{}\n性格：{}\n外貌：{}\n背景：{}\n目标：{}\n关系：{}\n当前状态：{}\n备注：{}",
        c.name,
        c.aliases,
        c.role,
        c.personality,
        c.appearance,
        c.background,
        c.goals,
        c.relationships,
        c.current_state,
        c.notes
    )
}

fn entry_item(e: &Entry, score: f32) -> ContextItem {
    ContextItem {
        kind: "entry".into(),
        id: e.id.clone(),
        title: format!("条目 · {}（{}）", e.title, e.category),
        snippet: clip(&e.content, 360),
        score,
    }
}

fn historical_chapters<'a>(bundle: &'a ProjectBundle, current: Option<&Chapter>) -> Vec<&'a Chapter> {
    let mut list: Vec<&Chapter> = bundle
        .chapters
        .iter()
        .filter(|c| !c.content.trim().is_empty() || !c.summary.trim().is_empty())
        .filter(|c| current.is_none_or(|cur| c.id != cur.id))
        .collect();
    list.sort_by(|a, b| a.order_index.cmp(&b.order_index).then(a.created_at.cmp(&b.created_at)));
    if list.is_empty() {
        bundle.chapters.iter().collect()
    } else {
        list
    }
}

fn chapter_extract(ch: &Chapter, max_chars: usize) -> String {
    if !ch.summary.trim().is_empty() {
        format!("{}\n{}", ch.summary, clip(&ch.content, max_chars.saturating_sub(ch.summary.chars().count())))
    } else {
        clip(&ch.content, max_chars)
    }
}

fn previous_summaries(bundle: &ProjectBundle, current: &Chapter) -> String {
    historical_chapters(bundle, Some(current))
        .into_iter()
        .filter(|c| {
            c.order_index < current.order_index
                || (c.order_index == current.order_index && c.created_at < current.created_at)
        })
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|c| format!("《{}》：{}", c.title, chapter_extract(c, 180)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn push_unique(items: &mut Vec<ContextItem>, item: ContextItem) {
    if items.iter().any(|x| x.kind == item.kind && x.id == item.id) {
        return;
    }
    items.push(item);
}

fn trim_budget(items: &mut Vec<ContextItem>, budget: usize) {
    let mut used = 0usize;
    items.retain(|item| {
        let cost = item.snippet.chars().count() + item.title.chars().count();
        if used + cost > budget && used > 0 {
            false
        } else {
            used += cost;
            true
        }
    });
}

fn build_system(bundle: &ProjectBundle, settings: &AppSettings, req: &GenerateRequest, target: &str) -> String {
    let style = if settings.writing_style.trim().is_empty() {
        bundle.project.style_prompt.clone()
    } else {
        format!(
            "{}\n{}",
            settings.writing_style.trim(),
            bundle.project.style_prompt
        )
    };
    let style = if style.trim().is_empty() {
        "跟随已有章节的语气与节奏。"
    } else {
        style.trim()
    };

    let shared = format!(
        "你是长篇小说创作助手「墨衡」。作者与你轮流改结构化对象：你起草或扩写，作者在中间栏修订，你再根据修订稿扩展。\n\
         书名：《{}》\n体裁：{}\n简介：{}\n写作风格：{}",
        bundle.project.title, bundle.project.genre, bundle.project.description, style
    );

    let rules = match (target, req.mode.as_str()) {
        ("memories", _) | (_, "memory") => {
            "记忆规则：\n\
             1. 记忆只能从已经写下的历史章节里提炼，禁止用大纲、设定或想象补未发生的事。\n\
             2. 每条必须是可核对的事实：谁做了什么、世界规则如何被证实、人物状态变成了什么。\n\
             3. 已有记忆不要重复，只补新事实。\n\
             4. 只输出 JSON 数组，每项含 kind, title, content, source。kind 只能是 plot、character、world。source 写成章名。"
        }
        ("characters", _) => {
            "角色规则：\n\
             1. 遵守已有记忆、章节事实与其他角色关系，不得和已发生情节冲突。\n\
             2. 若下方有「人工修订稿」，必须原样保留作者写过的事实，只补充空白或明显单薄的字段。\n\
             3. 只输出一个 JSON 对象，字段：name, aliases, role, personality, appearance, background, goals, relationships, currentState, notes。\n\
             4. 不要输出小说正文或解释。"
        }
        ("entries", _) => {
            "条目规则：\n\
             1. 条目是可检索的设定（地点、组织、物品、规则、事件、概念）。\n\
             2. 若有人工修订稿，保留作者已写事实，只扩展说明，不发明与章节/记忆冲突的新规则。\n\
             3. 只输出一个 JSON 对象，字段：title, category, content, tags。\n\
             4. category 只能是：地点、组织、物品、规则、事件、概念、设定。"
        }
        ("outlines", _) => {
            "大纲规则：\n\
             1. 大纲写节拍与必须发生的事，不要写成章节正文。\n\
             2. 若有人工修订稿，保留作者定下的情节点，只把含糊处写具体。\n\
             3. 只输出一个 JSON 对象，字段：title, content。"
        }
        (_, "summarize") => {
            "只根据本章已写正文做第三人称客观摘要，200字以内，供后文检索。只输出摘要，不发明未写情节。"
        }
        _ => {
            "章节规则：\n\
             1. 不得改写角色性格、外貌关键特征、世界规则、专有名词与已发生情节。\n\
             2. 若有人工修订稿或正文，扩写/续写必须承接作者改过的句子，不推翻不复述。\n\
             3. 只写小说正文，不要输出解释、标题或“接下来可以……”。\n\
             4. 文风克制、具体、可感。"
        }
    };

    format!("{shared}\n\n{rules}")
}

fn build_user(
    bundle: &ProjectBundle,
    req: &GenerateRequest,
    target: &str,
    chapter: Option<&Chapter>,
    outline: Option<&Outline>,
    character: Option<&Character>,
    entry: Option<&Entry>,
    items: &[ContextItem],
) -> String {
    let mut blocks = String::new();
    for item in items {
        blocks.push_str(&format!("## {}\n{}\n\n", item.title, item.snippet));
    }

    let snapshot = req.object_snapshot.as_deref().map(str::trim).and_then(|s| {
        if s.is_empty() || s == "{}" || s == "null" {
            None
        } else {
            Some(s.to_string())
        }
    });

    let revision = match target {
        "characters" => snapshot
            .clone()
            .or_else(|| character.map(format_character))
            .unwrap_or_else(|| "（当前角色还是空的，请起草。）".into()),
        "entries" => snapshot.clone().unwrap_or_else(|| {
            entry
                .map(|e| format!("标题：{}\n类别：{}\n标签：{}\n正文：{}", e.title, e.category, e.tags, e.content))
                .unwrap_or_else(|| "（当前条目还是空的，请起草。）".into())
        }),
        "outlines" => snapshot.clone().unwrap_or_else(|| {
            outline
                .map(|o| format!("标题：{}\n正文：{}", o.title, o.content))
                .unwrap_or_else(|| "（当前大纲还是空的，请起草。）".into())
        }),
        "chapters" => snapshot.clone().unwrap_or_else(|| {
            chapter
                .map(|c| {
                    format!(
                        "章名：{}\n摘要：{}\n正文：\n{}",
                        c.title,
                        if c.summary.is_empty() { "（无）" } else { &c.summary },
                        tail(&c.content, 2200)
                    )
                })
                .unwrap_or_else(|| "（当前章节还是空的，请起草。）".into())
        }),
        _ => snapshot.unwrap_or_default(),
    };

    let selected = req
        .selected_text
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("选中文本：\n{s}"))
        .unwrap_or_default();

    let task = match (target, req.mode.as_str()) {
        (_, "memory") | ("memories", _) => {
            "只从上面的历史章节提炼新的长期记忆。没有章节正文就返回 []。"
        }
        ("characters", "expand") => "这是作者改过的角色。保留已写事实，把单薄字段写具体。只输出 JSON。",
        ("characters", "rewrite") => "按用户指令改这份角色，仍只输出 JSON。",
        ("characters", _) => "根据书稿、记忆与其他对象起草这个角色。只输出 JSON。",
        ("entries", "expand") => "这是作者改过的条目。保留已写设定，只扩展说明。只输出 JSON。",
        ("entries", "rewrite") => "按用户指令改这份条目，仍只输出 JSON。",
        ("entries", _) => "根据书稿、记忆与章节事实起草这个条目。只输出 JSON。",
        ("outlines", "expand") => "这是作者改过的大纲。保留已定节拍，把含糊处写具体。只输出 JSON。",
        ("outlines", "rewrite") => "按用户指令改这份大纲，仍只输出 JSON。",
        ("outlines", _) => "根据角色、条目与记忆起草这份大纲。只输出 JSON，不要写成小说。",
        (_, "expand") => "在作者改过的正文上扩写：加感官与反应，少推进新情节，不发明新设定。",
        (_, "rewrite") => "按用户指令改写，保持人物与已发生事实一致。只输出正文。",
        (_, "summarize") => "客观摘要本章已写事实。只输出摘要。",
        (_, "draft") => "按大纲、角色、条目与记忆写本章初稿。只输出正文。",
        _ => "承接最后一句续写，不要复述已写内容。",
    };

    let instruction = if req.instruction.trim().is_empty() {
        match (target, req.mode.as_str()) {
            (_, "memory") => "提炼尚未入库的关键事实。".into(),
            ("chapters", "continue") => "按现有节奏续写 800 到 1200 字。".into(),
            ("chapters", "draft") => "写出本章可用的初稿，800 到 1500 字。".into(),
            (_, "expand") => "在人工修订稿上扩展，不要另起一套。".into(),
            (_, "draft") => "写出一份可给作者改的初稿。".into(),
            _ => "按任务要求处理。".into(),
        }
    } else {
        req.instruction.trim().to_string()
    };

    format!(
        "以下是本书的结构化对象、长期记忆与检索结果，生成时必须遵守。\n\n\
         {blocks}\n\
         ## 人工修订稿 / 当前对象\n{revision}\n\n\
         {selected}\n\n\
         对象：{}\n任务：{}\n任务说明：{task}\n用户指令：{instruction}\n书中已有角色：{}",
        target_label(target),
        mode_label(&req.mode),
        bundle
            .characters
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join("、")
    )
}

fn target_label(target: &str) -> &str {
    match target {
        "characters" => "角色",
        "entries" => "条目",
        "outlines" => "大纲",
        "memories" => "记忆",
        _ => "章节",
    }
}

fn mode_label(mode: &str) -> &str {
    match mode {
        "draft" => "起草",
        "rewrite" => "改写",
        "expand" => "扩写",
        "summarize" => "提炼摘要",
        "memory" => "从历史章节提炼记忆",
        _ => "续写",
    }
}

pub fn book_meta_messages(seed: &BookMetaSeed) -> Vec<ChatMessage> {
    let keep = if seed.mode == "expand" {
        "已填写的字段是作者修订稿，必须保留其事实与语气，只补空白或把一句话简介、文风写得更可用。不要另起一本完全不同的书。"
    } else {
        "已填写的字段优先沿用；空白字段由你起草。书名要像能印在封面的中文题名，不要翻译腔。"
    };
    let system = format!(
        "你是长篇小说策划助手「墨衡」。开书时先定书名、体裁、文风提示和一句话简介。\n\
         {keep}\n\
         只输出一个 JSON 对象，字段必须是：title, genre, stylePrompt, description。\n\
         genre 用短词，如「都市奇幻」「历史」「科幻」「世情」。\n\
         stylePrompt 写给后续章节生成用，短、可执行，例如「冷、短句、少解释」。\n\
         description 一句说清谁要什么、世界有什么不对。不要输出解释。"
    );
    let user = format!(
        "已填书名：{}\n已填体裁：{}\n已填文风：{}\n已填简介：{}\n作者指令：{}",
        empty_mark(&seed.title),
        empty_mark(&seed.genre),
        empty_mark(&seed.style_prompt),
        empty_mark(&seed.description),
        if seed.instruction.trim().is_empty() {
            "（无额外指令，按已填内容或自拟一本有辨识度的长篇。）"
        } else {
            seed.instruction.trim()
        }
    );
    vec![
        ChatMessage {
            role: "system".into(),
            content: system,
        },
        ChatMessage {
            role: "user".into(),
            content: user,
        },
    ]
}

fn empty_mark(text: &str) -> &str {
    let t = text.trim();
    if t.is_empty() {
        "（空）"
    } else {
        t
    }
}

fn tail(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "（尚无正文）".into();
    }
    let count = trimmed.chars().count();
    if count <= max_chars {
        return trimmed.to_string();
    }
    let skip = count - max_chars;
    format!("……{}", trimmed.chars().skip(skip).collect::<String>())
}
