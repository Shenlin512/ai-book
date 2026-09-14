# 墨衡 · AI 长篇小说创作

本地桌面应用。把长篇拆成可检索的结构化对象（书稿、角色、条目、大纲、章节、记忆），生成时按预算注入上下文，让后文仍认得前文。

作者与模型轮流改同一份对象：模型起草或扩写，作者在中间栏修订，模型再根据修订稿继续。密钥、书稿和模型请求都在本机，不经过第三方中转。

## 设计原则

- **对象即正文**：角色、条目、大纲、章节都是持续写作的对象，而不是一次性填表。
- **记忆只来自已发生的事**：长期记忆只能从已写章节提炼，禁止用大纲或设定补未发生情节。
- **修订稿优先**：扩写必须承接作者改过的句子与字段，不推翻、不复述、不另起一套。
- **上下文有预算**：检索、记忆、当前大纲、提及角色按优先级装入窗口，超出 `contextBudget` 截断。
- **接口统一**：在线与本地模型都走 OpenAI 兼容的 `/v1/chat/completions`。

## 技术栈

| 层 | 选型 |
| --- | --- |
| 桌面壳 | [Tauri 2](https://v2.tauri.app/) |
| 前端 | Vue 3 + TypeScript + Vite + Pinia + Vue Router |
| 后端 | Rust（`src-tauri`） |
| 存储 | SQLite（`rusqlite`，WAL，外键级联删除） |
| 模型调用 | `reqwest`，SSE 流式 / 非流式 JSON |

产品标识：`com.aibook.mohe`，窗口标题「墨衡 · AI 长篇创作」。

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│  Vue 前端（src/）                                            │
│  HomeView / WorkspaceView / SettingsView                    │
│  Pinia: workspace · settings                                │
│  api.ts → invoke / listen                                   │
└───────────────────────────┬─────────────────────────────────┘
                            │ Tauri IPC
                            │ 命令 + 事件 llm-token / llm-done / llm-error
┌───────────────────────────▼─────────────────────────────────┐
│  Rust 后端（src-tauri/src）                                  │
│  commands.rs  命令入口                                      │
│  db.rs        SQLite CRUD                                   │
│  prompt.rs    上下文装配与提示词                             │
│  retrieval.rs 中文 n-gram 检索                              │
│  llm.rs       OpenAI 兼容 Chat Completions                  │
│  seed.rs      示例书稿《烬城夜行》                           │
└───────────────┬──────────────────────────────┬──────────────┘
                │                              │
                ▼                              ▼
     本机 SQLite: mohe.db            在线 / 本地 LLM
     （应用数据目录）                 DeepSeek / Ollama / …
```

### 前后端边界

前端只负责界面、编辑态和把生成结果写回表单。所有持久化、检索、提示词拼装、模型调用都在 Rust 侧完成。

- 前端通过 `@tauri-apps/api` 的 `invoke` 调命令。
- 流式生成用事件：`llm-token`（逐 token）、`llm-done`（完成/取消）、`llm-error`。
- `cancel_generate` 置位进程内 `AtomicBool`，流读取循环检测到后停止。

### 数据落盘

应用启动时在 Tauri `app_data_dir` 下打开 `mohe.db`：

- Windows：`%APPDATA%\com.aibook.mohe\mohe.db`
- macOS：`~/Library/Application Support/com.aibook.mohe/mohe.db`
- Linux：`~/.local/share/com.aibook.mohe/mohe.db`

设置（含 API Key）以 JSON 存在 `settings` 表，不上传。主题（日课 / 夜读）存在浏览器 `localStorage` 的 `mohe-theme`。

### 目录结构

```
ai-book/
├── src/                      # Vue 前端
│   ├── api.ts                # IPC 封装与空对象工厂
│   ├── types.ts              # 与 Rust 模型对应的 TS 类型
│   ├── router.ts             # Hash 路由
│   ├── stores/
│   │   ├── workspace.ts      # 当前书稿包、编辑草稿、保存
│   │   └── settings.ts       # 模型预设与配置
│   ├── views/
│   │   ├── HomeView.vue      # 书稿列表 / 开书
│   │   ├── WorkspaceView.vue # 工作台三栏
│   │   └── SettingsView.vue  # 模型与上下文
│   └── components/
│       ├── GeneratePanel.vue # 生成、预览上下文、流式写入
│       └── BookMetaAi.vue    # 书名/体裁/文风/简介 AI 起草
├── src-tauri/src/
│   ├── lib.rs                # 启动、AppState、命令注册
│   ├── commands.rs
│   ├── db.rs
│   ├── models.rs
│   ├── prompt.rs
│   ├── retrieval.rs
│   ├── llm.rs
│   └── seed.rs
└── src-tauri/tauri.conf.json
```

## 数据模型

一本书是一个 `Project`。子对象全部带 `project_id`，删除书稿时级联删除。

```
Project 书稿
  ├── Character 角色（姓名、别称、定位、性格、外貌、背景、目标、关系、当前状态）
  ├── Entry     条目（地点 / 组织 / 物品 / 规则 / 事件 / 概念 / 设定）
  ├── Outline   大纲（可挂 parent_id，状态：草稿 / 在写 / 定稿）
  ├── Chapter   章节（可挂 outline_id，正文 + 摘要 + 状态）
  └── Memory    记忆（kind：plot / character / world / style）
```

加载工作台时一次取出 `ProjectBundle`（书稿 + 五类对象），供检索和提示词使用。

章节与大纲的状态：`draft` 草稿、`writing` 在写、`done` 定稿。

## 生成方案

核心问题：长篇上下文远超窗口，且作者会不断改模型输出。方案是 **结构化对象 + 轻量检索 + 有预算的提示词 + 修订稿回灌**。

### 流水线

```
GenerateRequest
    │
    ├─ 1. 加载 ProjectBundle + AppSettings
    ├─ 2. 由当前对象、指令、选区、修订稿快照组成检索 query
    ├─ 3. 中文 n-gram 检索 Top-K（可关）
    ├─ 4. 强制注入：当前大纲、勾选/提及角色、条目、记忆、前文摘要
    ├─ 5. 按 score 排序，按 contextBudget 截断
    ├─ 6. 拼 system / user 提示词
    └─ 7. 流式调用模型 → 写回对应对象
```

工作台右侧可先「预览上下文」，看到即将注入的条目和完整提示词，再生成。

### 检索（`retrieval.rs`）

不做向量库，避免本地再跑 embedding。

- 中文按 1/2/3-gram 切，英文按词切，过滤停用词。
- 文档来自当前书稿的角色、条目、大纲、章节（摘要+正文）、记忆。
- 标题加权；角色名 / 别称 / 条目标题出现在 query 里额外加分。
- 分数过低（≤ 0.35）丢弃，再取 `retrieveK`（默认 8）。

### 上下文装配（`prompt.rs`）

优先级（高分先入窗，超预算从尾部丢掉）：

| 来源 | 何时注入 | 典型权重 |
| --- | --- | --- |
| 当前大纲 | `includeOutline` | 99 |
| 当前角色 / 条目 | 工作台正打开该对象 | 99 |
| 用户勾选的角色、条目 | 生成面板多选 | 98 / 97 |
| 前文摘要 | 章节模式，取顺序靠前的若干章 | 95 |
| query 中点名的角色 | 姓名或别称命中 | 90 |
| 长期记忆 | `includeMemories`，最多 16 条 | 80 |
| 检索结果 | `includeRetrieval`，记忆提炼模式关闭检索 | 模型打分 |
| 历史章节全文摘录 | 仅「从章节提炼记忆」 | 96 起递减 |

记忆模式不走检索，只喂已写章节，避免把大纲里的「将要发生」写进记忆。

### 提示词分工

- **System**：书名、体裁、简介、文风（全局 `writingStyle` + 本书 `stylePrompt`）+ 当前对象的硬规则。
- **User**：检索块 + 「人工修订稿 / 当前对象」快照 + 选中文本 + 任务说明 + 用户指令。

对象不同，输出约束不同：

| 对象 | 模型输出 | 写回 |
| --- | --- | --- |
| 章节 | 纯小说正文 | 覆盖 / 接续 / 替换选区 |
| 章节摘要 | 第三人称客观摘要，约 200 字内 | 写入 `chapter.summary` |
| 角色 / 条目 / 大纲 | 单个 JSON 对象 | 解析后覆盖表单字段 |
| 记忆 | JSON 数组（kind, title, content, source） | 逐条插入 `memories` |
| 开书元数据 | JSON（title, genre, stylePrompt, description） | 填入书稿表单 |

章节流式生成时，token 边到边写进编辑器。结构化对象等整段结束后再解析 JSON。

### 人机协作循环

1. **起草**：对象为空或几乎为空时，按书稿与上下文生成初稿并覆盖。
2. **作者改**：在中间栏改字段或正文；`objectSnapshot` 记下当前稿。
3. **扩写**：提示词要求保留作者已写事实，只补空白或加感官，不另起炉灶。
4. **改写**：按指令改；章节若有选中文字，只替换选区。
5. **续写**（仅章节）：接到现有正文后，默认约 800–1200 字。
6. **摘要**：只根据本章已写正文，供后续检索，不发明未写情节。
7. **提炼记忆**：从历史章节抽出可核对事实，已有记忆不重复。

## 功能模块

### 1. 书稿管理

路由 `/`。列表展示本机全部书稿；可删除（连同子对象）。

开书可手填：书名、体裁、文风提示、一句话简介。也可让模型起草或按已填字段扩写（`generate_book_meta`）。内置示例书稿《烬城夜行》（都市奇幻），用于演示角色、条目、大纲、章节与记忆如何互相引用。

### 2. 工作台

路由 `/project/:id`。三栏：

- **左**：对象列表。页签：章节、大纲、角色、条目、记忆。
- **中**：当前对象编辑器。脏数据切换页签或条目时自动保存。
- **右**：生成面板。模式、指令、强制带上的角色/条目、上下文预览、流式输出、取消。

顶栏可改本书元数据（与开书同一套 AI 起草）。

### 3. 章节

正文、摘要、状态、关联大纲。生成模式：起草、续写、扩写、改写、摘要。

改写时若编辑器有选区，只替换选中段落。摘要成功后自动写回 `summary`，供后文章节检索「前文摘要」。

### 4. 大纲

写节拍与必须发生的事，不写成章节正文。可挂父节点、排序、状态。生成只输出 `{ title, content }`。

### 5. 角色

姓名、别称、定位、性格、外貌、背景、目标、关系、当前状态、备注。生成只输出对应 JSON。别称参与检索加权：正文或指令里出现名字/外号会强制带上该角色卡。

### 6. 条目（设定库）

类别：地点、组织、物品、规则、事件、概念、设定。带标签，供检索与生成勾选。

### 7. 记忆

四类：情节 `plot`、人物 `character`、世界 `world`、文风 `style`。可手记，也可从历史章节批量提炼。提炼结果自动入库；生成章节时默认注入最近记忆，约束性格、专名与已发生情节不得被改写。

### 8. 模型与上下文设置

路由 `/settings`。

**连接**

| 模式 | 预设 |
| --- | --- |
| 在线 | DeepSeek、OpenAI、通义千问、智谱 GLM、Kimi、自定义 |
| 本地 | Ollama（默认 `http://127.0.0.1:11434/v1`）、LM Studio、自定义 |

本地无密钥时请求头仍带 `Authorization: Bearer local`，以兼容部分 OpenAI 兼容服务。

「测试连接」会请求 `/v1/models`（能列出则展示数量），再发一条极短 chat 探针。

**生成参数**

- `temperature`、`maxTokens`
- `retrieveK`：检索条数
- `contextBudget`：注入上下文的字符预算（默认 7000，下限 1200）
- 开关：记忆、检索、大纲、角色、条目
- 全局文风 `writingStyle`：与每本书的 `stylePrompt` 叠加

### 9. 示例数据

`seed_demo_project` 若尚无标题为「烬城夜行」的书稿，则写入一套完整演示（角色沈照 / 白疏、地点与规则条目、大纲与章节、记忆）。已存在则直接打开。

## IPC 命令

| 命令 | 作用 |
| --- | --- |
| `list_projects` / `upsert_project` / `delete_project` | 书稿 |
| `get_project_bundle` | 加载整本书 |
| `upsert_*` / `delete_*` | 角色、条目、大纲、章节、记忆 |
| `get_settings` / `save_settings` | 本机配置 |
| `test_model` | 连通性 |
| `seed_demo_project` | 示例书稿 |
| `preview_context` | 只装配提示词，不调用模型 |
| `generate_text` | 流式生成 |
| `generate_book_meta` | 开书元数据（非流式） |
| `cancel_generate` | 中止当前流 |

## 本地运行

需要：Node.js、Rust 稳定版、[Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
npm install
npm run tauri dev
```

仅预览前端（无 SQLite / 模型命令）：

```bash
npm run dev
```

打包：

```bash
npm run tauri build
```

使用本地模型时，先启动 Ollama 或 LM Studio，再在设置里选对应预设并「测试连接」。

推荐编辑器扩展：Vue - Official、Tauri、rust-analyzer（见 `.vscode/extensions.json`）。

## 当前边界

- 检索是词面匹配，不是语义向量；同义改写可能召不回。
- 一次只服务一个生成流（全局 `cancel` 标志）。
- 大纲树在数据层支持 `parent_id`，界面以列表+排序为主。
- 无云同步、无多人协作；备份即复制 `mohe.db`。
