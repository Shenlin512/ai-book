use crate::models::*;

const STOP: &[&str] = &[
    "的", "了", "是", "在", "我", "有", "和", "就", "不", "人", "都", "一", "上", "也", "很",
    "到", "说", "要", "去", "你", "会", "着", "看", "好", "这", "那", "他", "她", "它", "们",
    "与", "及", "或", "并", "把", "被", "让", "给", "从", "向", "对", "为", "以", "而", "则",
    "又", "还", "已", "将", "能", "可", "却", "只", "再", "更", "最", "因为", "所以", "但是",
    "如果", "然后", "而且", "一个", "没有", "自己", "什么", "怎么", "这个", "那个", "the",
    "a", "an", "of", "to", "in", "on", "and", "or", "is", "are",
];

#[derive(Clone)]
struct Doc {
    kind: String,
    id: String,
    title: String,
    text: String,
}

pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut latin = String::new();
    let chars: Vec<char> = text.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if ch.is_ascii_alphanumeric() {
            latin.push(ch.to_ascii_lowercase());
            continue;
        }
        flush_latin(&mut latin, &mut tokens);
        if ch.is_whitespace() || is_punct(*ch) {
            continue;
        }
        let s = ch.to_string();
        if !is_stop(&s) {
            tokens.push(s);
        }
        if i + 1 < chars.len() {
            let n2: String = chars[i..=i + 1].iter().collect();
            if n2.chars().all(|c| !c.is_ascii() && !c.is_whitespace() && !is_punct(c)) && !is_stop(&n2)
            {
                tokens.push(n2);
            }
        }
        if i + 2 < chars.len() {
            let n3: String = chars[i..=i + 2].iter().collect();
            if n3.chars().all(|c| !c.is_ascii() && !c.is_whitespace() && !is_punct(c)) {
                tokens.push(n3);
            }
        }
    }
    flush_latin(&mut latin, &mut tokens);
    tokens
}

fn flush_latin(latin: &mut String, tokens: &mut Vec<String>) {
    if latin.len() >= 2 && !is_stop(latin) {
        tokens.push(latin.clone());
    }
    latin.clear();
}

fn is_punct(ch: char) -> bool {
    matches!(
        ch,
        '，' | '。' | '、' | '；' | '：' | '！' | '？' | '“' | '”' | '‘' | '’' | '（' | '）'
            | '《' | '》' | '—' | '…' | ',' | '.' | ';' | ':' | '!' | '?' | '"' | '\'' | '('
            | ')' | '[' | ']' | '{' | '}' | '-' | '/' | '\\'
    )
}

fn is_stop(token: &str) -> bool {
    STOP.contains(&token)
}

fn score(query: &[String], text: &str, title: &str) -> f32 {
    if query.is_empty() {
        return 0.0;
    }
    let doc_tokens = tokenize(&format!("{title} {title} {text}"));
    if doc_tokens.is_empty() {
        return 0.0;
    }
    let mut hits = 0.0;
    for q in query {
        let count = doc_tokens.iter().filter(|t| *t == q).count() as f32;
        if count > 0.0 {
            let weight = if q.chars().count() >= 2 { 1.6 } else { 0.45 };
            hits += (1.0 + count.ln()) * weight;
        }
    }
    hits / (1.0 + (doc_tokens.len() as f32).sqrt() * 0.08)
}

pub fn search(bundle: &ProjectBundle, query: &str, limit: usize) -> Vec<ContextItem> {
    let tokens = tokenize(query);
    let mut docs = Vec::new();

    for c in &bundle.characters {
        docs.push(Doc {
            kind: "character".into(),
            id: c.id.clone(),
            title: format!("角色 · {}", c.name),
            text: [
                c.name.as_str(),
                c.aliases.as_str(),
                c.role.as_str(),
                c.personality.as_str(),
                c.appearance.as_str(),
                c.background.as_str(),
                c.goals.as_str(),
                c.relationships.as_str(),
                c.current_state.as_str(),
                c.notes.as_str(),
            ]
            .join("\n"),
        });
    }
    for e in &bundle.entries {
        docs.push(Doc {
            kind: "entry".into(),
            id: e.id.clone(),
            title: format!("条目 · {}", e.title),
            text: format!("{}\n{}\n{}", e.category, e.tags, e.content),
        });
    }
    for o in &bundle.outlines {
        docs.push(Doc {
            kind: "outline".into(),
            id: o.id.clone(),
            title: format!("大纲 · {}", o.title),
            text: o.content.clone(),
        });
    }
    for ch in &bundle.chapters {
        docs.push(Doc {
            kind: "chapter".into(),
            id: ch.id.clone(),
            title: format!("章节 · {}", ch.title),
            text: format!("{}\n{}", ch.summary, ch.content),
        });
    }
    for m in &bundle.memories {
        docs.push(Doc {
            kind: "memory".into(),
            id: m.id.clone(),
            title: format!("记忆 · {}", m.title),
            text: format!("{}\n{}", m.kind, m.content),
        });
    }

    let mut scored: Vec<ContextItem> = docs
        .into_iter()
        .map(|doc| {
            let mut s = score(&tokens, &doc.text, &doc.title);
            for name_hit in name_boosts(bundle, query) {
                if doc.id == name_hit {
                    s += 3.5;
                }
            }
            ContextItem {
                kind: doc.kind,
                id: doc.id,
                title: doc.title,
                snippet: clip(&doc.text, 280),
                score: s,
            }
        })
        .filter(|item| item.score > 0.35)
        .collect();

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored
}

fn name_boosts(bundle: &ProjectBundle, query: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for c in &bundle.characters {
        if query.contains(&c.name) {
            ids.push(c.id.clone());
            continue;
        }
        for alias in c.aliases.split(|ch| matches!(ch, ',' | '，' | ';' | '；' | '、' | ' ')) {
            let alias = alias.trim();
            if !alias.is_empty() && query.contains(alias) {
                ids.push(c.id.clone());
                break;
            }
        }
    }
    for e in &bundle.entries {
        if query.contains(&e.title) {
            ids.push(e.id.clone());
        }
    }
    ids
}

pub fn clip(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    let count = trimmed.chars().count();
    if count <= max_chars {
        return trimmed.to_string();
    }
    format!("{}…", trimmed.chars().take(max_chars).collect::<String>())
}
