<script setup lang="ts">
import { onMounted, ref } from "vue"
import { RouterLink, RouterView, useRoute } from "vue-router"
import { useSettings } from "./stores/settings"

const route = useRoute()
const settings = useSettings()
const theme = ref<"light" | "dark">("light")

onMounted(async () => {
  const saved = localStorage.getItem("mohe-theme")
  if (saved === "dark" || saved === "light") theme.value = saved
  document.documentElement.dataset.theme = theme.value
  try {
    await settings.load()
  } catch {
    // 设置在桌面端初始化；浏览器预览可稍后配置
  }
})

function toggleTheme() {
  theme.value = theme.value === "light" ? "dark" : "light"
  document.documentElement.dataset.theme = theme.value
  localStorage.setItem("mohe-theme", theme.value)
}
</script>

<template>
  <div class="app-shell">
    <header class="topbar">
      <RouterLink class="brand" to="/">
        <div class="seal">衡</div>
        <div>
          <h1>墨衡</h1>
          <small>长篇一致的持续写作</small>
        </div>
      </RouterLink>
      <div class="top-actions">
        <span class="muted" v-if="settings.current">
          {{ settings.current.mode === "local" ? "本地" : "在线" }} ·
          {{ settings.current.model || "未选模型" }}
        </span>
        <button class="btn ghost" type="button" @click="toggleTheme">
          {{ theme === "light" ? "夜读" : "日课" }}
        </button>
        <RouterLink class="btn" to="/settings">设置</RouterLink>
        <RouterLink v-if="route.name !== 'home'" class="btn ghost" to="/">全部书稿</RouterLink>
      </div>
    </header>
    <RouterView />
  </div>
</template>
