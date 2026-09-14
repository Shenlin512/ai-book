export interface Project {
  id: string
  title: string
  genre: string
  description: string
  stylePrompt: string
  createdAt: string
  updatedAt: string
}

export interface Character {
  id: string
  projectId: string
  name: string
  aliases: string
  role: string
  personality: string
  appearance: string
  background: string
  goals: string
  relationships: string
  currentState: string
  notes: string
  createdAt: string
  updatedAt: string
}

export interface Entry {
  id: string
  projectId: string
  title: string
  category: string
  content: string
  tags: string
  createdAt: string
  updatedAt: string
}

export interface Outline {
  id: string
  projectId: string
  parentId: string | null
  title: string
  content: string
  orderIndex: number
  status: string
  createdAt: string
  updatedAt: string
}

export interface Chapter {
  id: string
  projectId: string
  outlineId: string | null
  title: string
  content: string
  summary: string
  orderIndex: number
  status: string
  createdAt: string
  updatedAt: string
}

export interface Memory {
  id: string
  projectId: string
  kind: string
  title: string
  content: string
  source: string
  createdAt: string
  updatedAt: string
}

export interface ProjectBundle {
  project: Project
  characters: Character[]
  entries: Entry[]
  outlines: Outline[]
  chapters: Chapter[]
  memories: Memory[]
}

export interface AppSettings {
  mode: "online" | "local" | string
  preset: string
  baseUrl: string
  apiKey: string
  model: string
  temperature: number
  maxTokens: number
  retrieveK: number
  contextBudget: number
  includeMemories: boolean
  includeRetrieval: boolean
  includeOutline: boolean
  includeCharacters: boolean
  includeEntries: boolean
  writingStyle: string
}

export interface ContextItem {
  kind: string
  id: string
  title: string
  snippet: string
  score: number
}

export interface ContextPreview {
  items: ContextItem[]
  systemPrompt: string
  userPrompt: string
}

export interface ConnectionTest {
  ok: boolean
  message: string
  models: string[]
}

export interface BookMeta {
  title: string
  genre: string
  stylePrompt: string
  description: string
}

export interface BookMetaSeed extends BookMeta {
  instruction: string
  mode: "draft" | "expand" | string
}

export interface GenerateRequest {
  projectId: string
  chapterId?: string | null
  outlineId?: string | null
  characterId?: string | null
  entryId?: string | null
  characterIds: string[]
  entryIds: string[]
  instruction: string
  mode: GenerateMode
  target: WorkspaceTab
  selectedText?: string | null
  objectSnapshot?: string | null
}

export type GenerateMode =
  | "draft"
  | "continue"
  | "expand"
  | "rewrite"
  | "summarize"
  | "memory"

export type WorkspaceTab = "characters" | "entries" | "outlines" | "chapters" | "memories"

export const ENTRY_CATEGORIES = ["地点", "组织", "物品", "规则", "事件", "概念", "设定"] as const

export const MEMORY_KINDS = [
  { id: "plot", label: "情节" },
  { id: "character", label: "人物" },
  { id: "world", label: "世界" },
  { id: "style", label: "文风" },
] as const

export const STATUS_OPTIONS = [
  { id: "draft", label: "草稿" },
  { id: "writing", label: "在写" },
  { id: "done", label: "定稿" },
] as const
