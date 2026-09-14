import { defineStore } from "pinia"
import { computed, ref } from "vue"
import {
  api,
  emptyChapter,
  emptyCharacter,
  emptyEntry,
  emptyMemory,
  emptyOutline,
} from "../api"
import type {
  Chapter,
  Character,
  Entry,
  GenerateMode,
  Memory,
  Outline,
  Project,
  ProjectBundle,
  WorkspaceTab,
} from "../types"

export const useWorkspace = defineStore("workspace", () => {
  const bundle = ref<ProjectBundle | null>(null)
  const tab = ref<WorkspaceTab>("chapters")
  const selectedId = ref<string>("")
  const loading = ref(false)
  const saving = ref(false)
  const dirty = ref(false)
  const error = ref("")
  const snapshot = ref("")
  const workingCharacter = ref<Character>(emptyCharacter(""))
  const workingEntry = ref<Entry>(emptyEntry(""))
  const workingOutline = ref<Outline>(emptyOutline("", null, 1))
  const workingChapter = ref<Chapter>(emptyChapter("", 1))
  const workingMemory = ref<Memory>(emptyMemory(""))

  const project = computed(() => bundle.value?.project ?? null)
  const characters = computed(() => bundle.value?.characters ?? [])
  const entries = computed(() => bundle.value?.entries ?? [])
  const outlines = computed(() => bundle.value?.outlines ?? [])
  const chapters = computed(() => bundle.value?.chapters ?? [])
  const memories = computed(() => bundle.value?.memories ?? [])

  async function load(id: string) {
    loading.value = true
    error.value = ""
    try {
      bundle.value = await api.getBundle(id)
      if (!selectedId.value) {
        selectedId.value = chapters.value[0]?.id ?? characters.value[0]?.id ?? ""
        tab.value = chapters.value.length ? "chapters" : "characters"
      }
      hydrateWorking()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
    } finally {
      loading.value = false
    }
  }

  async function reload() {
    if (bundle.value) await load(bundle.value.project.id)
  }

  function select(nextTab: WorkspaceTab, id: string) {
    tab.value = nextTab
    selectedId.value = id
    hydrateWorking()
  }

  function hydrateWorking() {
    const projectId = project.value?.id ?? ""
    const id = selectedId.value
    workingCharacter.value =
      characters.value.find((item) => item.id === id) ?? emptyCharacter(projectId)
    workingEntry.value = entries.value.find((item) => item.id === id) ?? emptyEntry(projectId)
    workingOutline.value =
      outlines.value.find((item) => item.id === id) ??
      emptyOutline(projectId, null, outlines.value.length + 1)
    workingChapter.value =
      chapters.value.find((item) => item.id === id) ?? emptyChapter(projectId, chapters.value.length + 1)
    workingMemory.value = memories.value.find((item) => item.id === id) ?? emptyMemory(projectId)
    dirty.value = false
    touchSnapshot()
  }

  function currentWorking() {
    if (tab.value === "characters") return workingCharacter.value
    if (tab.value === "entries") return workingEntry.value
    if (tab.value === "outlines") return workingOutline.value
    if (tab.value === "memories") return workingMemory.value
    return workingChapter.value
  }

  function touchSnapshot() {
    snapshot.value = JSON.stringify(currentWorking())
  }

  function markDirty() {
    dirty.value = true
    touchSnapshot()
  }

  async function saveProject(projectValue: Project) {
    saving.value = true
    try {
      const saved = await api.upsertProject(projectValue)
      if (bundle.value) bundle.value.project = saved
      return saved
    } finally {
      saving.value = false
    }
  }

  async function saveCharacter(item: Character) {
    const saved = await api.upsertCharacter(item)
    replace("characters", saved)
    selectedId.value = saved.id
    workingCharacter.value = saved
    return saved
  }

  async function saveEntry(item: Entry) {
    const saved = await api.upsertEntry(item)
    replace("entries", saved)
    selectedId.value = saved.id
    workingEntry.value = saved
    return saved
  }

  async function saveOutline(item: Outline) {
    const saved = await api.upsertOutline(item)
    replace("outlines", saved)
    selectedId.value = saved.id
    workingOutline.value = saved
    return saved
  }

  async function saveChapter(item: Chapter) {
    const saved = await api.upsertChapter(item)
    replace("chapters", saved)
    selectedId.value = saved.id
    workingChapter.value = saved
    return saved
  }

  async function saveMemory(item: Memory) {
    const saved = await api.upsertMemory(item)
    replace("memories", saved)
    selectedId.value = saved.id
    workingMemory.value = saved
    return saved
  }

  async function saveWorking() {
    saving.value = true
    try {
      if (project.value) await saveProject(project.value)
      if (tab.value === "characters" && workingCharacter.value.name.trim()) {
        await saveCharacter(workingCharacter.value)
      } else if (tab.value === "entries" && workingEntry.value.title.trim()) {
        await saveEntry(workingEntry.value)
      } else if (tab.value === "outlines" && workingOutline.value.title.trim()) {
        await saveOutline(workingOutline.value)
      } else if (tab.value === "chapters" && workingChapter.value.title.trim()) {
        await saveChapter(workingChapter.value)
      } else if (tab.value === "memories" && workingMemory.value.title.trim()) {
        await saveMemory(workingMemory.value)
      }
      dirty.value = false
      touchSnapshot()
    } finally {
      saving.value = false
    }
  }

  async function removeCurrent() {
    if (!selectedId.value) return
    const id = selectedId.value
    if (tab.value === "characters") await api.deleteCharacter(id)
    if (tab.value === "entries") await api.deleteEntry(id)
    if (tab.value === "outlines") await api.deleteOutline(id)
    if (tab.value === "chapters") await api.deleteChapter(id)
    if (tab.value === "memories") await api.deleteMemory(id)
    await reload()
    selectedId.value = currentList().value[0]?.id ?? ""
    hydrateWorking()
  }

  function currentList() {
    if (tab.value === "characters") return characters
    if (tab.value === "entries") return entries
    if (tab.value === "outlines") return outlines
    if (tab.value === "memories") return memories
    return chapters
  }

  function setSnapshot(data: unknown) {
    snapshot.value = JSON.stringify(data)
  }

  function replace<K extends "characters" | "entries" | "outlines" | "chapters" | "memories">(
    key: K,
    item: ProjectBundle[K][number],
  ) {
    if (!bundle.value) return
    const list = bundle.value[key] as Array<{ id: string }>
    const idx = list.findIndex((row) => row.id === item.id)
    if (idx >= 0) list[idx] = item
    else list.unshift(item)
  }

  function writeChapterStream(
    mode: GenerateMode,
    prefix: string,
    text: string,
    selectedText?: string | null,
    baseContent?: string,
  ) {
    const chapter = workingChapter.value
    if (mode === "summarize") {
      chapter.summary = text
    } else if (mode === "continue") {
      chapter.content = prefix + text
    } else if (mode === "rewrite" && selectedText?.trim() && (baseContent ?? "").includes(selectedText)) {
      chapter.content = (baseContent ?? "").replace(selectedText, text)
    } else {
      chapter.content = text
    }
    markDirty()
  }

  function writeStructured(data: Record<string, string>, raw: string) {
    if (tab.value === "characters") {
      const base = workingCharacter.value
      workingCharacter.value = {
        ...base,
        name: data.name || base.name || "未命名角色",
        aliases: data.aliases ?? base.aliases,
        role: data.role ?? base.role,
        personality: data.personality ?? base.personality,
        appearance: data.appearance ?? base.appearance,
        background: data.background ?? base.background,
        goals: data.goals ?? base.goals,
        relationships: data.relationships ?? base.relationships,
        currentState: data.currentState ?? data.current_state ?? base.currentState,
        notes: data.notes ?? base.notes,
      }
    } else if (tab.value === "entries") {
      const base = workingEntry.value
      workingEntry.value = {
        ...base,
        title: data.title || base.title || "未命名条目",
        category: data.category || base.category,
        tags: data.tags ?? base.tags,
        content: data.content || raw,
      }
    } else if (tab.value === "outlines") {
      const base = workingOutline.value
      workingOutline.value = {
        ...base,
        title: data.title || base.title || "未命名节点",
        content: data.content || raw,
      }
    }
    markDirty()
  }

  return {
    bundle,
    tab,
    selectedId,
    loading,
    saving,
    dirty,
    error,
    snapshot,
    project,
    characters,
    entries,
    outlines,
    chapters,
    memories,
    workingCharacter,
    workingEntry,
    workingOutline,
    workingChapter,
    workingMemory,
    load,
    reload,
    select,
    hydrateWorking,
    saveProject,
    saveCharacter,
    saveEntry,
    saveOutline,
    saveChapter,
    saveMemory,
    saveWorking,
    removeCurrent,
    setSnapshot,
    markDirty,
    touchSnapshot,
    writeChapterStream,
    writeStructured,
  }
})
