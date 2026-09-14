import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import type {
  AppSettings,
  Chapter,
  Character,
  BookMeta,
  BookMetaSeed,
  ConnectionTest,
  ContextPreview,
  Entry,
  GenerateRequest,
  Memory,
  Outline,
  Project,
  ProjectBundle,
} from "./types"

export const api = {
  listProjects: () => invoke<Project[]>("list_projects"),
  getBundle: (id: string) => invoke<ProjectBundle>("get_project_bundle", { id }),
  upsertProject: (project: Project) => invoke<Project>("upsert_project", { project }),
  deleteProject: (id: string) => invoke<void>("delete_project", { id }),
  upsertCharacter: (item: Character) => invoke<Character>("upsert_character", { item }),
  deleteCharacter: (id: string) => invoke<void>("delete_character", { id }),
  upsertEntry: (item: Entry) => invoke<Entry>("upsert_entry", { item }),
  deleteEntry: (id: string) => invoke<void>("delete_entry", { id }),
  upsertOutline: (item: Outline) => invoke<Outline>("upsert_outline", { item }),
  deleteOutline: (id: string) => invoke<void>("delete_outline", { id }),
  upsertChapter: (item: Chapter) => invoke<Chapter>("upsert_chapter", { item }),
  deleteChapter: (id: string) => invoke<void>("delete_chapter", { id }),
  upsertMemory: (item: Memory) => invoke<Memory>("upsert_memory", { item }),
  deleteMemory: (id: string) => invoke<void>("delete_memory", { id }),
  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>("save_settings", { settings }),
  testModel: () => invoke<ConnectionTest>("test_model"),
  seedDemo: () => invoke<Project>("seed_demo_project"),
  previewContext: (request: GenerateRequest) =>
    invoke<ContextPreview>("preview_context", { request }),
  generateText: (request: GenerateRequest) => invoke<string>("generate_text", { request }),
  generateBookMeta: (seed: BookMetaSeed) => invoke<BookMeta>("generate_book_meta", { seed }),
  cancelGenerate: () => invoke<void>("cancel_generate"),
}

export async function listenGeneration(handlers: {
  onToken: (token: string) => void
  onDone: (payload: { text: string; cancelled: boolean }) => void
  onError: (message: string) => void
}): Promise<UnlistenFn> {
  const offs = await Promise.all([
    listen<{ token: string }>("llm-token", (event) => handlers.onToken(event.payload.token)),
    listen<{ text: string; cancelled: boolean }>("llm-done", (event) => handlers.onDone(event.payload)),
    listen<string>("llm-error", (event) => handlers.onError(event.payload)),
  ])
  return () => {
    offs.forEach((off) => off())
  }
}

export function emptyProject(): Project {
  return {
    id: "",
    title: "",
    genre: "",
    description: "",
    stylePrompt: "",
    createdAt: "",
    updatedAt: "",
  }
}

export function emptyCharacter(projectId: string): Character {
  return {
    id: "",
    projectId,
    name: "",
    aliases: "",
    role: "",
    personality: "",
    appearance: "",
    background: "",
    goals: "",
    relationships: "",
    currentState: "",
    notes: "",
    createdAt: "",
    updatedAt: "",
  }
}

export function emptyEntry(projectId: string): Entry {
  return {
    id: "",
    projectId,
    title: "",
    category: "设定",
    content: "",
    tags: "",
    createdAt: "",
    updatedAt: "",
  }
}

export function emptyOutline(projectId: string, parentId: string | null, orderIndex: number): Outline {
  return {
    id: "",
    projectId,
    parentId,
    title: "",
    content: "",
    orderIndex,
    status: "draft",
    createdAt: "",
    updatedAt: "",
  }
}

export function emptyChapter(projectId: string, orderIndex: number): Chapter {
  return {
    id: "",
    projectId,
    outlineId: null,
    title: "",
    content: "",
    summary: "",
    orderIndex,
    status: "draft",
    createdAt: "",
    updatedAt: "",
  }
}

export function emptyMemory(projectId: string): Memory {
  return {
    id: "",
    projectId,
    kind: "plot",
    title: "",
    content: "",
    source: "手记",
    createdAt: "",
    updatedAt: "",
  }
}

export function countWords(text: string): number {
  return text.replace(/\s+/g, "").length
}
