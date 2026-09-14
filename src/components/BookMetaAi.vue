<script setup lang="ts">
import { ref } from "vue"
import { api } from "../api"
import type { BookMeta } from "../types"

const props = defineProps<{
  title: string
  genre: string
  stylePrompt: string
  description: string
  compact?: boolean
}>()

const emit = defineEmits<{
  apply: [meta: BookMeta]
}>()

const instruction = ref("")
const busy = ref(false)
const error = ref("")

async function generate(mode: "draft" | "expand") {
  error.value = ""
  busy.value = true
  try {
    const meta = await api.generateBookMeta({
      title: props.title,
      genre: props.genre,
      stylePrompt: props.stylePrompt,
      description: props.description,
      instruction: instruction.value,
      mode,
    })
    emit("apply", meta)
  } catch (err) {
    error.value = String(err)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="book-meta-ai" :class="{ compact }">
    <div class="field">
      <label>开书指令（可空）</label>
      <textarea
        v-model="instruction"
        :rows="compact ? 2 : 3"
        placeholder="想写一本什么样的书。空着则按已填内容起草，或自拟书名、体裁、文风和简介。"
      />
    </div>
    <div class="hero-actions" style="justify-content: flex-start; margin-bottom: 10px">
      <button class="btn primary" type="button" :disabled="busy" @click="generate('draft')">
        {{ busy ? "生成中…" : "AI 起草" }}
      </button>
      <button class="btn" type="button" :disabled="busy" @click="generate('expand')">按已填扩写</button>
    </div>
    <p v-if="error" class="toast">{{ error }}</p>
  </div>
</template>
