<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import { api } from "../api"
import { PRESETS, useSettings } from "../stores/settings"
import type { AppSettings, ConnectionTest } from "../types"

const settings = useSettings()
const draft = ref<AppSettings | null>(null)
const testing = ref(false)
const result = ref<ConnectionTest | null>(null)
const saved = ref("")

const onlinePresets = computed(() => PRESETS.filter((item) => item.mode === "online"))
const localPresets = computed(() => PRESETS.filter((item) => item.mode === "local"))

onMounted(async () => {
  const current = settings.current ?? (await settings.load())
  draft.value = { ...current }
})

function setMode(mode: "online" | "local") {
  if (!draft.value) return
  draft.value.mode = mode
  const first = PRESETS.find((item) => item.mode === mode)
  if (first) applyPreset(first.id)
}

function applyPreset(id: string) {
  if (!draft.value) return
  draft.value = settings.applyPreset(id, draft.value)
}

async function persist() {
  if (!draft.value) return
  await settings.save(draft.value)
  saved.value = "已写入本地配置"
}

async function test() {
  if (!draft.value) return
  testing.value = true
  saved.value = ""
  await settings.save(draft.value)
  try {
    result.value = await api.testModel()
  } catch (err) {
    result.value = { ok: false, message: String(err), models: [] }
  } finally {
    testing.value = false
  }
}
</script>

<template>
  <main class="page" v-if="draft">
    <section class="hero">
      <div>
        <h2>模型与上下文</h2>
        <p>在线接口和本地接口都走 OpenAI 兼容的 `/v1/chat/completions`。密钥只存在本机数据库，不会上传。</p>
      </div>
      <div class="row" style="flex: 0 0 auto">
        <button class="btn" type="button" :disabled="testing" @click="test">
          {{ testing ? "正在试连…" : "测试连接" }}
        </button>
        <button class="btn primary" type="button" @click="persist">保存设置</button>
      </div>
    </section>

    <p v-if="saved" class="muted">{{ saved }}</p>
    <p v-if="result" class="toast" :style="{ color: result.ok ? 'var(--moss)' : 'var(--cinnabar-deep)' }">
      {{ result.message }}
    </p>

    <section class="settings-grid">
      <aside>
        <button class="choice" :class="{ active: draft.mode === 'online' }" type="button" @click="setMode('online')">
          <b>在线模型</b>
          <p class="muted">DeepSeek、通义、OpenAI 等云端接口。</p>
        </button>
        <button class="choice" :class="{ active: draft.mode === 'local' }" type="button" @click="setMode('local')">
          <b>本地模型</b>
          <p class="muted">Ollama、LM Studio 或任意本机 OpenAI 兼容服务。</p>
        </button>
        <div class="chips">
          <button
            v-for="item in draft.mode === 'local' ? localPresets : onlinePresets"
            :key="item.id"
            class="chip"
            :class="{ hot: draft.preset === item.id }"
            type="button"
            @click="applyPreset(item.id)"
          >
            {{ item.label }}
          </button>
        </div>
      </aside>

      <section class="card">
        <div class="field">
          <label>接口地址</label>
          <input v-model="draft.baseUrl" placeholder="https://api.deepseek.com/v1 或 http://127.0.0.1:11434/v1" />
        </div>
        <div class="row">
          <div class="field">
            <label>模型名</label>
            <input v-model="draft.model" placeholder="deepseek-chat / qwen2.5:14b" />
          </div>
          <div class="field">
            <label>{{ draft.mode === "local" ? "密钥（本地通常可空）" : "API Key" }}</label>
            <input v-model="draft.apiKey" type="password" autocomplete="off" />
          </div>
        </div>
        <div class="row">
          <div class="field">
            <label>温度 {{ draft.temperature.toFixed(2) }}</label>
            <input v-model.number="draft.temperature" type="range" min="0" max="1.5" step="0.05" />
          </div>
          <div class="field">
            <label>最大生成长度</label>
            <input v-model.number="draft.maxTokens" type="number" min="256" max="8192" />
          </div>
        </div>
        <div class="row">
          <div class="field">
            <label>检索条数</label>
            <input v-model.number="draft.retrieveK" type="number" min="3" max="20" />
          </div>
          <div class="field">
            <label>上下文预算（字）</label>
            <input v-model.number="draft.contextBudget" type="number" min="1500" max="20000" />
          </div>
        </div>
        <div class="field">
          <label>全局文风（会与书稿文风叠加）</label>
          <textarea v-model="draft.writingStyle" rows="3" placeholder="例如：少形容词，对话推动场面，不解释人物动机。" />
        </div>
        <div class="chips">
          <label class="chip"><input v-model="draft.includeCharacters" type="checkbox" /> 注入角色</label>
          <label class="chip"><input v-model="draft.includeEntries" type="checkbox" /> 注入条目</label>
          <label class="chip"><input v-model="draft.includeOutline" type="checkbox" /> 注入大纲</label>
          <label class="chip"><input v-model="draft.includeMemories" type="checkbox" /> 注入记忆</label>
          <label class="chip"><input v-model="draft.includeRetrieval" type="checkbox" /> 关键词检索</label>
        </div>
        <p class="muted" style="margin-top: 16px">
          本地示例：先启动 Ollama，拉好模型后把地址设为 `http://127.0.0.1:11434/v1`，模型名写成你本地的标签，如 `qwen2.5:14b`。
        </p>
        <div v-if="result?.models.length" class="chips" style="margin-top: 12px">
          <button
            v-for="name in result.models.slice(0, 16)"
            :key="name"
            class="chip"
            type="button"
            @click="draft.model = name"
          >
            {{ name }}
          </button>
        </div>
      </section>
    </section>
  </main>
</template>
