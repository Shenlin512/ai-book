<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue"
import { api, listenGeneration } from "../api"
import { useWorkspace } from "../stores/workspace"
import type { ContextPreview, GenerateMode, GenerateRequest, WorkspaceTab } from "../types"

const workspace = useWorkspace()
const instruction = ref("")
const mode = ref<GenerateMode>("draft")
const preview = ref<ContextPreview | null>(null)
const output = ref("")
const busy = ref(false)
const error = ref("")
const hint = ref("")
const selectedCharacterIds = ref<string[]>([])
const selectedEntryIds = ref<string[]>([])
let streamPrefix = ""
let streamBase = ""
let selectedForRewrite = ""
let stopListen: (() => void) | null = null

const modes = computed(() => {
  const tab = workspace.tab
  if (tab === "memories") {
    return [
      {
        id: "memory" as const,
        label: "从章节提炼",
        hint: "只根据已写的历史章节抽出长期事实，不编造",
      },
    ]
  }
  if (tab === "chapters") {
    return [
      { id: "draft" as const, label: "起草", hint: "按大纲、角色与记忆写本章初稿，覆盖正文" },
      { id: "continue" as const, label: "续写", hint: "接到现有正文后面" },
      { id: "expand" as const, label: "扩写", hint: "用扩写后的正文覆盖本章" },
      { id: "rewrite" as const, label: "改写", hint: "覆盖正文；若有选中文字则只替换选中部分" },
      { id: "summarize" as const, label: "摘要", hint: "覆盖本章摘要栏" },
    ]
  }
  const noun = tab === "characters" ? "角色" : tab === "entries" ? "条目" : "大纲"
  return [
    { id: "draft" as const, label: "起草", hint: `生成${noun}并直接覆盖表单` },
    { id: "expand" as const, label: "扩写", hint: `在你改过的${noun}上补充，再覆盖表单` },
    { id: "rewrite" as const, label: "改写", hint: `按指令改这份${noun}并覆盖表单` },
  ]
})

watch(
  () => workspace.tab,
  (tab) => {
    const next = modes.value[0]?.id
    if (next) mode.value = next
    if (tab === "chapters" && workspace.workingChapter.content.trim()) {
      mode.value = "continue"
    } else if (tab !== "memories" && hasDraftContent(tab)) {
      mode.value = "expand"
    }
  },
)

function hasDraftContent(tab: WorkspaceTab): boolean {
  if (tab === "characters") {
    return Boolean(workspace.workingCharacter.personality || workspace.workingCharacter.background)
  }
  if (tab === "entries") return Boolean(workspace.workingEntry.content.trim())
  if (tab === "outlines") return Boolean(workspace.workingOutline.content.trim())
  if (tab === "chapters") return Boolean(workspace.workingChapter.content.trim())
  return false
}

function buildRequest(): GenerateRequest {
  return {
    projectId: workspace.project?.id ?? "",
    chapterId: workspace.tab === "chapters" ? workspace.workingChapter.id || workspace.selectedId : null,
    outlineId:
      workspace.tab === "outlines"
        ? workspace.workingOutline.id
        : workspace.workingChapter.outlineId,
    characterId: workspace.tab === "characters" ? workspace.workingCharacter.id : null,
    entryId: workspace.tab === "entries" ? workspace.workingEntry.id : null,
    characterIds: selectedCharacterIds.value,
    entryIds: selectedEntryIds.value,
    instruction: instruction.value,
    mode: mode.value,
    target: workspace.tab,
    selectedText: selectedForRewrite || window.getSelection()?.toString() || null,
    objectSnapshot: workspace.snapshot || JSON.stringify(currentSnapshot()),
  }
}

function currentSnapshot() {
  if (workspace.tab === "characters") return workspace.workingCharacter
  if (workspace.tab === "entries") return workspace.workingEntry
  if (workspace.tab === "outlines") return workspace.workingOutline
  if (workspace.tab === "memories") return workspace.workingMemory
  return workspace.workingChapter
}

onMounted(async () => {
  stopListen = await listenGeneration({
    onToken: (token) => {
      output.value += token
      if (workspace.tab === "chapters" && mode.value !== "memory") {
        workspace.writeChapterStream(mode.value, streamPrefix, output.value, selectedForRewrite, streamBase)
      }
    },
    onDone: () => {
      busy.value = false
      finishApply()
    },
    onError: (message) => {
      busy.value = false
      error.value = message
    },
  })
  await refreshPreview()
})

onUnmounted(() => {
  stopListen?.()
})

watch(
  () => [workspace.selectedId, workspace.tab, mode.value],
  () => {
    void refreshPreview()
  },
)

async function refreshPreview() {
  if (!workspace.project) return
  try {
    preview.value = await api.previewContext(buildRequest())
  } catch (err) {
    error.value = String(err)
  }
}

async function generate() {
  if (!workspace.project) return
  error.value = ""
  hint.value = ""
  output.value = ""
  selectedForRewrite = window.getSelection()?.toString() || ""
  streamBase = workspace.workingChapter.content
  if (workspace.tab === "chapters") {
    if (mode.value === "continue") {
      const body = workspace.workingChapter.content.trim()
      streamPrefix = body ? `${body}\n\n` : ""
    } else {
      streamPrefix = ""
    }
  }
  busy.value = true
  workspace.touchSnapshot()
  await refreshPreview()
  try {
    await api.generateText(buildRequest())
    if (mode.value === "memory") {
      await workspace.reload()
      hint.value = "已从历史章节写入记忆库。"
    }
  } catch (err) {
    error.value = String(err)
    busy.value = false
  }
}

function finishApply() {
  if (mode.value === "memory" || mode.value === "summarize") {
    if (mode.value === "summarize") hint.value = "摘要已写入表单，请点保存。"
    return
  }
  if (workspace.tab === "chapters") {
    hint.value =
      mode.value === "continue" ? "续写已接到正文后，请点保存。" : "正文已覆盖到表单，请点保存。"
    return
  }
  const data = parseJson(output.value)
  if (data || output.value.trim()) {
    workspace.writeStructured(data ?? {}, output.value.trim())
    hint.value = "已覆盖到表单，改完后请点保存。"
  }
}

function parseJson(text: string): Record<string, string> | null {
  const start = text.indexOf("{")
  const end = text.lastIndexOf("}")
  if (start < 0 || end <= start) return null
  try {
    const value = JSON.parse(text.slice(start, end + 1)) as Record<string, unknown>
    const result: Record<string, string> = {}
    for (const [key, raw] of Object.entries(value)) {
      if (typeof raw === "string") result[key] = raw
    }
    return result
  } catch {
    return null
  }
}

async function cancel() {
  await api.cancelGenerate()
  busy.value = false
}

function toggle(list: string[], id: string) {
  const idx = list.indexOf(id)
  if (idx >= 0) list.splice(idx, 1)
  else list.push(id)
}
</script>

<template>
  <aside class="ai-pane pane">
    <div class="toolbar">
      <div>
        <strong>墨衡</strong>
      </div>
      <button class="btn ghost" type="button" @click="refreshPreview">刷新上下文</button>
    </div>
    <p class="muted">
      生成结果会直接写入中间表单。续写接到正文后；起草、扩写、改写覆盖对应栏。改完请点保存。
    </p>

    <div class="chips" style="margin-bottom: 12px">
      <button
        v-for="item in modes"
        :key="item.id"
        class="chip"
        :class="{ hot: mode === item.id }"
        type="button"
        :title="item.hint"
        @click="mode = item.id"
      >
        {{ item.label }}
      </button>
    </div>

    <div class="field">
      <label>指令</label>
      <textarea
        v-model="instruction"
        rows="4"
        :placeholder="(modes.find((item) => item.id === mode)?.hint ?? '') + '。空着则按默认任务写。'"
      />
    </div>

    <div v-if="workspace.tab !== 'memories'" class="field">
      <label>额外点名角色</label>
      <div class="chips">
        <button
          v-for="item in workspace.characters"
          :key="item.id"
          class="chip"
          :class="{ hot: selectedCharacterIds.includes(item.id) }"
          type="button"
          @click="toggle(selectedCharacterIds, item.id)"
        >
          {{ item.name }}
        </button>
        <span v-if="!workspace.characters.length" class="muted">还没有角色。</span>
      </div>
    </div>

    <div v-if="workspace.tab !== 'memories'" class="field">
      <label>额外点名条目</label>
      <div class="chips">
        <button
          v-for="item in workspace.entries"
          :key="item.id"
          class="chip"
          :class="{ hot: selectedEntryIds.includes(item.id) }"
          type="button"
          @click="toggle(selectedEntryIds, item.id)"
        >
          {{ item.title }}
        </button>
        <span v-if="!workspace.entries.length" class="muted">还没有条目。</span>
      </div>
    </div>

    <div class="field">
      <label>将注入的上下文</label>
      <div v-if="preview?.items.length" class="context">
        <details v-for="item in preview.items" :key="item.kind + item.id">
          <summary>{{ item.title }} · {{ item.score.toFixed(1) }}</summary>
          <p class="muted">{{ item.snippet }}</p>
        </details>
      </div>
      <p v-else class="muted">
        {{
          workspace.tab === "memories"
            ? "会把已写章节和已有记忆送进模型，只抽新事实。"
            : "当前表单、相关设定和检索结果会出现在这里。"
        }}
      </p>
    </div>

    <div class="row" style="margin-bottom: 12px">
      <button class="btn primary" type="button" :disabled="busy || !workspace.project" @click="generate">
        {{ busy ? "写入表单中…" : mode === "memory" ? "提炼记忆" : mode === "draft" ? "起草" : "开始生成" }}
      </button>
      <button class="btn" type="button" :disabled="!busy" @click="cancel">停下</button>
    </div>

    <p v-if="error" class="toast">{{ error }}</p>
    <p v-if="hint" class="muted">{{ hint }}</p>
    <p v-if="busy && workspace.tab === 'chapters'" class="muted">正在把字写进正文栏…</p>
  </aside>
</template>
