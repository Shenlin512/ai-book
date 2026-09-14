<script setup lang="ts">
import { onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import BookMetaAi from "../components/BookMetaAi.vue"
import { api, emptyProject } from "../api"
import type { BookMeta, Project } from "../types"

const router = useRouter()
const projects = ref<Project[]>([])
const error = ref("")
const creating = ref(false)
const draft = ref(emptyProject())
const savedHint = ref("")

onMounted(load)

async function load() {
  try {
    projects.value = await api.listProjects()
  } catch (err) {
    error.value = String(err)
  }
}

async function saveDraft() {
  if (!draft.value.title.trim()) {
    error.value = "先写一个书名。"
    return
  }
  error.value = ""
  draft.value = await api.upsertProject({ ...draft.value })
  savedHint.value = "书稿信息已保存"
  await load()
}

async function createProject() {
  await saveDraft()
  if (!draft.value.id) return
  await router.push(`/project/${draft.value.id}`)
}

async function openDemo() {
  const project = await api.seedDemo()
  await router.push(`/project/${project.id}`)
}

function applyMeta(meta: BookMeta) {
  draft.value = {
    ...draft.value,
    title: meta.title || draft.value.title,
    genre: meta.genre || draft.value.genre,
    stylePrompt: meta.stylePrompt || draft.value.stylePrompt,
    description: meta.description || draft.value.description,
  }
}

async function remove(project: Project, event: Event) {
  event.preventDefault()
  event.stopPropagation()
  if (!confirm(`删除《${project.title}》以及其中的角色、条目、大纲与章节？`)) return
  await api.deleteProject(project.id)
  await load()
}
</script>

<template>
  <main class="page">
    <section class="hero">
      <div class="hero-copy">
        <h2>把长篇写成可检索的世界</h2>
        <p>
          角色、条目、大纲、章节都是持续写作的对象。生成时会注入记忆与检索结果，让后文仍认得前文。
        </p>
      </div>
      <div class="hero-actions">
        <button class="btn" type="button" @click="openDemo">打开示例《烬城夜行》</button>
        <button class="btn primary" type="button" @click="creating = !creating">
          {{ creating ? "收起" : "新开书稿" }}
        </button>
      </div>
    </section>

    <p v-if="error" class="toast">{{ error }}</p>

    <section v-if="creating" class="card" style="margin-bottom: 22px; max-width: 720px">
      <BookMetaAi
        :title="draft.title"
        :genre="draft.genre"
        :style-prompt="draft.stylePrompt"
        :description="draft.description"
        @apply="applyMeta"
      />
      <div class="field">
        <label>书名</label>
        <input v-model="draft.title" placeholder="例如：烬城夜行" />
      </div>
      <div class="row">
        <div class="field">
          <label>体裁</label>
          <input v-model="draft.genre" placeholder="都市奇幻 / 历史 / 科幻" />
        </div>
        <div class="field">
          <label>文风提示</label>
          <input v-model="draft.stylePrompt" placeholder="冷、短句、少解释" />
        </div>
      </div>
      <div class="field">
        <label>一句话简介</label>
        <textarea v-model="draft.description" rows="3" placeholder="这本书在写什么，谁要得到什么。" />
      </div>
      <p v-if="savedHint" class="muted">{{ savedHint }}</p>
      <div class="hero-actions" style="justify-content: flex-start">
        <button class="btn" type="button" @click="saveDraft">保存</button>
        <button class="btn primary" type="button" @click="createProject">进入工作室</button>
      </div>
    </section>

    <section v-if="!projects.length" class="empty">
      还没有书稿。先开一本，或载入示例，看角色、记忆与检索如何一起工作。
    </section>

    <section class="grid-cards">
      <RouterLink v-for="project in projects" :key="project.id" class="card" :to="`/project/${project.id}`">
        <div class="meta">{{ project.genre || "未定体裁" }} · {{ project.updatedAt }}</div>
        <h3>{{ project.title }}</h3>
        <p class="muted">{{ project.description || "还没有简介。" }}</p>
        <div class="card-actions">
          <span class="chip">进入写作</span>
          <button class="btn ghost" type="button" @click="remove(project, $event)">删除</button>
        </div>
      </RouterLink>
    </section>
  </main>
</template>
