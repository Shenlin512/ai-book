<script setup lang="ts">
import { computed, onMounted, watch } from "vue"
import { useRoute } from "vue-router"
import BookMetaAi from "../components/BookMetaAi.vue"
import GeneratePanel from "../components/GeneratePanel.vue"
import { countWords, emptyChapter, emptyCharacter, emptyEntry, emptyMemory, emptyOutline } from "../api"
import { useWorkspace } from "../stores/workspace"
import {
  ENTRY_CATEGORIES,
  MEMORY_KINDS,
  STATUS_OPTIONS,
  type BookMeta,
  type WorkspaceTab,
} from "../types"

const route = useRoute()
const workspace = useWorkspace()
const tabs: { id: WorkspaceTab; label: string }[] = [
  { id: "chapters", label: "章节" },
  { id: "outlines", label: "大纲" },
  { id: "characters", label: "角色" },
  { id: "entries", label: "条目" },
  { id: "memories", label: "记忆" },
]

const list = computed(() => {
  if (workspace.tab === "characters") return workspace.characters
  if (workspace.tab === "entries") return workspace.entries
  if (workspace.tab === "outlines") return workspace.outlines
  if (workspace.tab === "memories") return workspace.memories
  return workspace.chapters
})

const saveLabel = computed(() => {
  if (workspace.saving) return "保存中…"
  if (workspace.dirty) return "保存"
  return "已保存"
})

onMounted(() => loadProject())
watch(() => route.params.id, () => loadProject())

async function loadProject() {
  const id = String(route.params.id || "")
  if (id) await workspace.load(id)
}

function onEdit() {
  workspace.markDirty()
}

async function switchTab(tab: WorkspaceTab) {
  if (workspace.dirty) await workspace.saveWorking()
  const first = (
    tab === "characters"
      ? workspace.characters
      : tab === "entries"
        ? workspace.entries
        : tab === "outlines"
          ? workspace.outlines
          : tab === "memories"
            ? workspace.memories
            : workspace.chapters
  )[0]
  workspace.select(tab, first?.id ?? "")
}

async function openItem(id: string) {
  if (workspace.dirty) await workspace.saveWorking()
  workspace.select(workspace.tab, id)
}

async function createItem() {
  if (workspace.dirty) await workspace.saveWorking()
  const projectId = workspace.project?.id
  if (!projectId) return
  if (workspace.tab === "characters") {
    const saved = await workspace.saveCharacter({
      ...emptyCharacter(projectId),
      name: `新角色 ${workspace.characters.length + 1}`,
    })
    workspace.select("characters", saved.id)
  }
  if (workspace.tab === "entries") {
    const saved = await workspace.saveEntry({
      ...emptyEntry(projectId),
      title: `新条目 ${workspace.entries.length + 1}`,
    })
    workspace.select("entries", saved.id)
  }
  if (workspace.tab === "outlines") {
    const saved = await workspace.saveOutline({
      ...emptyOutline(projectId, null, workspace.outlines.length + 1),
      title: `节点 ${workspace.outlines.length + 1}`,
    })
    workspace.select("outlines", saved.id)
  }
  if (workspace.tab === "chapters") {
    const saved = await workspace.saveChapter({
      ...emptyChapter(projectId, workspace.chapters.length + 1),
      title: `第${workspace.chapters.length + 1}章`,
    })
    workspace.select("chapters", saved.id)
  }
  if (workspace.tab === "memories") {
    const saved = await workspace.saveMemory({
      ...emptyMemory(projectId),
      title: "新记忆",
      content: "需要后续写作遵守的事实。",
    })
    workspace.select("memories", saved.id)
  }
}

function applyBookMeta(meta: BookMeta) {
  if (!workspace.project) return
  workspace.project.title = meta.title || workspace.project.title
  workspace.project.genre = meta.genre || workspace.project.genre
  workspace.project.stylePrompt = meta.stylePrompt || workspace.project.stylePrompt
  workspace.project.description = meta.description || workspace.project.description
  workspace.markDirty()
}

function itemTitle(item: { title?: string; name?: string }): string {
  return item.title || item.name || "未命名"
}

function itemMeta(item: {
  role?: string
  category?: string
  status?: string
  kind?: string
  summary?: string
}): string {
  return item.role || item.category || item.status || item.kind || item.summary || ""
}
</script>

<template>
  <div v-if="workspace.loading" class="page">正在展开书案…</div>
  <div v-else-if="!workspace.project" class="page">找不到这本册子。</div>
  <div v-else class="workspace">
    <nav class="side-nav pane">
      <BookMetaAi
        compact
        :title="workspace.project.title"
        :genre="workspace.project.genre"
        :style-prompt="workspace.project.stylePrompt"
        :description="workspace.project.description"
        @apply="applyBookMeta"
      />
      <div class="field">
        <label>书名</label>
        <input v-model="workspace.project.title" @input="onEdit" />
      </div>
      <div class="field">
        <label>一句话简介</label>
        <textarea v-model="workspace.project.description" rows="3" placeholder="这本书在写什么" @input="onEdit" />
      </div>
      <div class="field">
        <label>体裁</label>
        <input v-model="workspace.project.genre" placeholder="都市奇幻 / 历史 / 科幻" @input="onEdit" />
      </div>
      <div class="field">
        <label>文风提示</label>
        <textarea v-model="workspace.project.stylePrompt" rows="3" placeholder="文风会进入每次生成" @input="onEdit" />
      </div>
      <button class="btn primary" type="button" :disabled="workspace.saving || !workspace.dirty" @click="workspace.saveWorking()">
        {{ saveLabel }}
      </button>
      <button
        v-for="item in tabs"
        :key="item.id"
        type="button"
        :class="{ active: workspace.tab === item.id }"
        @click="switchTab(item.id)"
      >
        <span>{{ item.label }}</span>
        <span>{{
          item.id === "characters"
            ? workspace.characters.length
            : item.id === "entries"
              ? workspace.entries.length
              : item.id === "outlines"
                ? workspace.outlines.length
                : item.id === "memories"
                  ? workspace.memories.length
                  : workspace.chapters.length
        }}</span>
      </button>
      <p class="muted">生成会直接写入表单。续写追加，其他正文生成覆盖。改完点保存。</p>
    </nav>

    <section class="list-pane pane">
      <div class="list-head">
        <strong>{{ tabs.find((item) => item.id === workspace.tab)?.label }}</strong>
        <button class="btn" type="button" @click="createItem">新建</button>
      </div>
      <button
        v-for="item in list"
        :key="item.id"
        class="item"
        :class="{ active: item.id === workspace.selectedId }"
        type="button"
        @click="openItem(item.id)"
      >
        <b>{{ itemTitle(item) }}</b>
        <span>{{ itemMeta(item) }}</span>
      </button>
      <div v-if="!list.length" class="empty">还是空的。先建一条，再让模型围着它写。</div>
    </section>

    <section class="editor-pane pane">
      <div class="toolbar">
        <div class="muted">{{ workspace.dirty ? "有未保存的改动" : "表单与书稿信息已同步" }}</div>
        <div class="hero-actions">
          <button class="btn primary" type="button" :disabled="workspace.saving || !workspace.dirty" @click="workspace.saveWorking()">
            {{ saveLabel }}
          </button>
          <button class="btn ghost" type="button" :disabled="!workspace.selectedId" @click="workspace.removeCurrent">
            删除当前
          </button>
        </div>
      </div>

      <template v-if="workspace.tab === 'characters'">
        <div class="row">
          <div class="field"><label>姓名</label><input v-model="workspace.workingCharacter.name" @input="onEdit" /></div>
          <div class="field"><label>别称</label><input v-model="workspace.workingCharacter.aliases" @input="onEdit" /></div>
        </div>
        <div class="field"><label>定位</label><input v-model="workspace.workingCharacter.role" @input="onEdit" /></div>
        <div class="field"><label>性格与说话方式</label><textarea v-model="workspace.workingCharacter.personality" rows="4" @input="onEdit" /></div>
        <div class="field"><label>外貌</label><textarea v-model="workspace.workingCharacter.appearance" rows="3" @input="onEdit" /></div>
        <div class="field"><label>背景</label><textarea v-model="workspace.workingCharacter.background" rows="4" @input="onEdit" /></div>
        <div class="field"><label>目标</label><textarea v-model="workspace.workingCharacter.goals" rows="3" @input="onEdit" /></div>
        <div class="field"><label>关系</label><textarea v-model="workspace.workingCharacter.relationships" rows="3" @input="onEdit" /></div>
        <div class="field"><label>当前状态（会随情节更新）</label><textarea v-model="workspace.workingCharacter.currentState" rows="3" @input="onEdit" /></div>
        <div class="field"><label>写作备注</label><textarea v-model="workspace.workingCharacter.notes" rows="3" @input="onEdit" /></div>
      </template>

      <template v-else-if="workspace.tab === 'entries'">
        <div class="row">
          <div class="field"><label>标题</label><input v-model="workspace.workingEntry.title" @input="onEdit" /></div>
          <div class="field">
            <label>类别</label>
            <select v-model="workspace.workingEntry.category" @change="onEdit">
              <option v-for="item in ENTRY_CATEGORIES" :key="item" :value="item">{{ item }}</option>
            </select>
          </div>
        </div>
        <div class="field"><label>标签</label><input v-model="workspace.workingEntry.tags" placeholder="用逗号分隔" @input="onEdit" /></div>
        <div class="field manuscript-field"><label>正文</label><textarea class="manuscript" v-model="workspace.workingEntry.content" rows="18" @input="onEdit" /></div>
      </template>

      <template v-else-if="workspace.tab === 'outlines'">
        <div class="row">
          <div class="field"><label>标题</label><input v-model="workspace.workingOutline.title" @input="onEdit" /></div>
          <div class="field">
            <label>状态</label>
            <select v-model="workspace.workingOutline.status" @change="onEdit">
              <option v-for="item in STATUS_OPTIONS" :key="item.id" :value="item.id">{{ item.label }}</option>
            </select>
          </div>
        </div>
        <div class="row">
          <div class="field">
            <label>父节点</label>
            <select v-model="workspace.workingOutline.parentId" @change="onEdit">
              <option :value="null">无（卷 / 部）</option>
              <option v-for="item in workspace.outlines.filter((row) => row.id !== workspace.workingOutline.id)" :key="item.id" :value="item.id">
                {{ item.title }}
              </option>
            </select>
          </div>
          <div class="field"><label>顺序</label><input v-model.number="workspace.workingOutline.orderIndex" type="number" @change="onEdit" /></div>
        </div>
        <div class="field manuscript-field"><label>节拍与必须发生的事</label><textarea class="manuscript" v-model="workspace.workingOutline.content" rows="16" @input="onEdit" /></div>
      </template>

      <template v-else-if="workspace.tab === 'memories'">
        <div class="row">
          <div class="field"><label>标题</label><input v-model="workspace.workingMemory.title" @input="onEdit" /></div>
          <div class="field">
            <label>类型</label>
            <select v-model="workspace.workingMemory.kind" @change="onEdit">
              <option v-for="item in MEMORY_KINDS" :key="item.id" :value="item.id">{{ item.label }}</option>
            </select>
          </div>
        </div>
        <div class="field"><label>来源</label><input v-model="workspace.workingMemory.source" @input="onEdit" /></div>
        <div class="field manuscript-field"><label>必须遵守的事实</label><textarea class="manuscript" v-model="workspace.workingMemory.content" rows="14" @input="onEdit" /></div>
      </template>

      <template v-else>
        <div class="row">
          <div class="field"><label>章名</label><input v-model="workspace.workingChapter.title" @input="onEdit" /></div>
          <div class="field">
            <label>状态</label>
            <select v-model="workspace.workingChapter.status" @change="onEdit">
              <option v-for="item in STATUS_OPTIONS" :key="item.id" :value="item.id">{{ item.label }}</option>
            </select>
          </div>
        </div>
        <div class="row">
          <div class="field">
            <label>绑定大纲</label>
            <select v-model="workspace.workingChapter.outlineId" @change="onEdit">
              <option :value="null">不绑定</option>
              <option v-for="item in workspace.outlines" :key="item.id" :value="item.id">{{ item.title }}</option>
            </select>
          </div>
          <div class="field"><label>顺序</label><input v-model.number="workspace.workingChapter.orderIndex" type="number" @change="onEdit" /></div>
        </div>
        <div class="field"><label>本章摘要（给后文检索用）</label><textarea v-model="workspace.workingChapter.summary" rows="3" @input="onEdit" /></div>
        <div class="toolbar">
          <span class="meta">{{ countWords(workspace.workingChapter.content) }} 字</span>
        </div>
        <div class="field manuscript-field">
          <textarea
            class="manuscript"
            v-model="workspace.workingChapter.content"
            @input="onEdit"
            placeholder="从这一页开始写。右侧生成会按规则直接写入这里。"
          />
        </div>
      </template>
    </section>

    <GeneratePanel />
  </div>
</template>
