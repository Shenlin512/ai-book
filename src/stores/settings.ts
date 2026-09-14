import { defineStore } from "pinia"
import { ref } from "vue"
import { api } from "../api"
import type { AppSettings } from "../types"

export const PRESETS = [
  {
    id: "deepseek",
    label: "DeepSeek",
    mode: "online",
    baseUrl: "https://api.deepseek.com/v1",
    model: "deepseek-chat",
  },
  {
    id: "openai",
    label: "OpenAI",
    mode: "online",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
  },
  {
    id: "qwen",
    label: "通义千问",
    mode: "online",
    baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    model: "qwen-plus",
  },
  {
    id: "zhipu",
    label: "智谱 GLM",
    mode: "online",
    baseUrl: "https://open.bigmodel.cn/api/paas/v4",
    model: "glm-4-flash",
  },
  {
    id: "moonshot",
    label: "Kimi",
    mode: "online",
    baseUrl: "https://api.moonshot.cn/v1",
    model: "moonshot-v1-auto",
  },
  {
    id: "custom-online",
    label: "自定义在线接口",
    mode: "online",
    baseUrl: "",
    model: "",
  },
  {
    id: "ollama",
    label: "Ollama",
    mode: "local",
    baseUrl: "http://127.0.0.1:11434/v1",
    model: "qwen2.5:14b",
  },
  {
    id: "lmstudio",
    label: "LM Studio",
    mode: "local",
    baseUrl: "http://127.0.0.1:1234/v1",
    model: "local-model",
  },
  {
    id: "custom-local",
    label: "自定义本地接口",
    mode: "local",
    baseUrl: "http://127.0.0.1:8080/v1",
    model: "",
  },
] as const

export const useSettings = defineStore("settings", () => {
  const current = ref<AppSettings | null>(null)
  const loaded = ref(false)

  async function load() {
    current.value = await api.getSettings()
    loaded.value = true
    return current.value
  }

  async function save(next: AppSettings) {
    current.value = await api.saveSettings(next)
    return current.value
  }

  function applyPreset(presetId: string, draft: AppSettings): AppSettings {
    const preset = PRESETS.find((item) => item.id === presetId)
    if (!preset) return { ...draft, preset: presetId }
    return {
      ...draft,
      preset: preset.id,
      mode: preset.mode,
      baseUrl: preset.baseUrl || draft.baseUrl,
      model: preset.model || draft.model,
    }
  }

  return { current, loaded, load, save, applyPreset }
})
