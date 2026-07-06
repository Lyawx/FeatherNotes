<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { computed } from 'vue';
import Titlebar from './components/TitleBar.vue';
import SvgIcon from './components/SvgIcon.vue';
import AiTab from './components/AiTab.vue';
import SettingsTab from './components/SettingsTab.vue';
import LogbookTab from "./components/LogbookTab.vue";
import TaskTab from "./components/TaskTab.vue";
import { mainService, type OllamaStatus } from "./services/mainService";

type TabName = 'ai' | 'settings' | 'logbook' | 'tasks';
const activeTab = ref<TabName>('ai');

// 1. L'objet contenant uniquement tes COMPOSANTS pour le <component :is>
const tabComponents: Record<TabName, any> = {
  ai: AiTab,
  settings: SettingsTab,
  logbook: LogbookTab,
  tasks: TaskTab
};

const currentTabComponent = computed(() => tabComponents[activeTab.value]);

const navigationTabs = [
  { id: 'ai', icon: 'ai' },
  { id: 'logbook', icon: 'logbook' },
  { id: 'tasks', icon: 'tasks' }
];

const ollamaStatus = ref<OllamaStatus>('connecting');
let statusInterval: number | null = null;

const checkHeartbeat = async () => {
  if (ollamaStatus.value === 'connecting') return;
  try {
    const isAlive = await mainService.ollama.checkStatus();
    ollamaStatus.value = isAlive ? 'connected' : 'disconnected';
  } catch {
    ollamaStatus.value = 'disconnected';
  }
};

const handleForcedStart = async () => {
  if (ollamaStatus.value !== 'disconnected') return;
  ollamaStatus.value = 'connecting';
  try {
    await mainService.ollama.startProcess();
    for (let i = 0; i < 20; i++) {
      await new Promise(resolve => setTimeout(resolve, 500));
      if (await mainService.ollama.checkStatus()) {
        ollamaStatus.value = 'connected';
        return;
      }
    }
    ollamaStatus.value = 'disconnected';
  } catch {
    ollamaStatus.value = 'disconnected';
  }
};

onMounted(async () => {
  await mainService.centerWindow();
  try {
    const isAlive = await mainService.ollama.checkStatus();
    ollamaStatus.value = isAlive ? 'connected' : 'disconnected';
  } catch {
    ollamaStatus.value = 'disconnected';
  }
  statusInterval = window.setInterval(checkHeartbeat, 5000);
});

onUnmounted(() => {
  if (statusInterval) clearInterval(statusInterval);
});
</script>

<template>
  <div id="app-root" class="app-layout">
    <Titlebar />
    <div id="app-wrapper">
      <aside class="sidebar-left">
      <nav class="nav-links">
        <div class="main-nav-group">
          <button v-for="tab in navigationTabs" :key="tab.id" :class="['nav-btn', { active: activeTab === tab.id }]"
            @click="activeTab = tab.id as TabName">
            <SvgIcon :name="tab.icon" class="tab-icon" />
          </button>
        </div>

        <button :class="['nav-btn', 'settings-nav-btn', { active: activeTab === 'settings' }]"
          @click="activeTab = 'settings'">
          <SvgIcon name="settings" class="tab-icon" />
        </button>
      </nav>
    </aside>

    <main class="main-content">
      <component :is="currentTabComponent" :ollamaStatus="ollamaStatus" @triggerStart="handleForcedStart"
        @updateStatus="ollamaStatus = $event" />
    </main>
    </div>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body,
html {
  width: 100vw;
  height: 100vh;
  background-color: var(--bg-00);
  overflow: hidden;
  font-family: Inter, system-ui, sans-serif;
  color-scheme: dark;
  color: var(--text-01);
  display: flex;
  flex-direction: column;
}

.app-layout {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

#app-wrapper {
  display: flex;
  flex-direction: row;
  flex: 1;
  min-height: 0; 
  overflow: hidden;
}

.main-content {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  background-color: var(--bg-00);
}

.sidebar-left {
  width: 70px;
  background-color: var(--bg-01);
  border-right: var(--border-width) solid var(--bg-02);
  padding: 1.5rem 0;
}

.nav-links {
  display: flex;
  flex-direction: column;
  height: 100%;
  align-items: center;
}

.main-nav-group {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: 100%;
  align-items: center;
}

.settings-nav-btn {
  margin-top: auto;
}

.nav-btn {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  border: var(--border-width) solid transparent;
  background-color: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  color: var(--text-02);
}

.nav-btn:hover {
  background-color: var(--bg-02);
  color: var(--text-00);
}

.nav-btn.active {
  background-color: var(--color-blue-dark);
  color: var(--text-00);
  border-color: var(--color-blue);
}

.tab-icon {
  width: 22px;
  height: 22px;
  fill: currentColor;
}

.tab-panel {
  padding: 2.5rem;
  min-height: 100%;
}

h1 {
  font-size: 1.6rem;
  font-weight: 500;
  color: var(--text-01);
  margin-bottom: 0.5rem;
  letter-spacing: -0.025em;
}

h2 {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--text-01);
}

h3 {
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--text-01);
}

p {
  font-size: 0.95rem;
  color: var(--text-01);
  margin-bottom: 1.5rem;
}

.tab-desc{
  color: var(--text-02);
  margin-bottom: 2rem;
}

.container-light {
  background-color: var(--bg-01);
  border: var(--border-width) solid var(--bg-02);
  border-radius: 8px;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  margin-bottom: 1.25rem;
}

.container-dark {
  background-color: var(--bg-00);
  border: var(--border-width) solid var(--bg-02);
  border-radius: 8px;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  margin-bottom: 1.25rem;
}

.ui-divider {
  height: var(--border-width);
  margin: none;
  padding: none;
  width: 100%;
  background-color: var(--bg-02);
  border: none;
}

.text-input-field {
  width: 100%;
  background-color: var(--bg-00);
  border: var(--border-width) solid var(--bg-02);
  color: var(--text-00);
  padding: 0.65rem 0.85rem;
  border-radius: 8px;
  font-family: inherit;
  font-size: 0.95rem;
  transition: all 0.2s;
}

.text-input-field:focus {
  outline: none;
  border-color: var(--color-blue);
}

.text-input-field:disabled {
  background-color: var(--bg-01);
  color: var(--text-02);
  cursor: not-allowed;
  border-color: var(--bg-02);
}

.base-btn {
  padding: 0.6rem 1.2rem;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 600;
  transition: all 0.2s;
  border: var(--border-width) solid var(--bg-02);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-00);
  color: var(--text-00);
}

.base-btn:disabled {
  background-color: var(--bg-01) !important;
  color: var(--text-02) !important;
  border-color: var(--bg-02) !important;
  box-shadow: none !important;
  cursor: not-allowed;
}

.base-btn:hover:not(:disabled) {
  background-color: var(--bg-02);
}

.green-btn {
  background-color: var(--color-green-surface);
  color: var(--color-green-hover);
  border-color: var(--color-green);
}

.green-btn:hover:not(:disabled) {
  background-color: var(--color-green);
  color: var(--bg-00);
}

.blue-btn {
  background-color: var(--color-blue-surface);
  color: var(--color-blue-hover);
  border-color: var(--color-blue);
}

.blue-btn:hover:not(:disabled) {
  background-color: var(--color-blue);
  color: var(--text-00);
}

.red-btn {
  background-color: var(--color-red-surface);
  color: var(--color-red-hover);
  border-color: var(--color-red);
}

.red-btn:hover:not(:disabled) {
  background-color: var(--color-red);
  color: var(--text-00);
}

.ui-toggle-wrapper {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.ui-toggle-wrapper input {
  opacity: 0;
  width: 0;
  height: 0;
}

.ui-toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-00);
  transition: .3s;
  border-radius: 24px;
  border: var(--border-width) solid var(--bg-02);
}

.ui-toggle-slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: var(--text-01);
  transition: .3s;
  border-radius: 50%;
}

input:checked+.ui-toggle-slider {
  background-color: var(--color-blue);
  border-color: var(--color-blue-dark);
}

input:checked+.ui-toggle-slider:before {
  transform: translateX(20px);
  background-color: var(--text-00);
}

.ui-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid var(--text-01);
  border-top-color: transparent;
  border-radius: 50%;
  animation: global-spin 1s linear infinite;
  display: inline-block;
}

@keyframes global-spin {
  to {
    transform: rotate(360deg);
  }
}

.ui-progress-container {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  align-items: center;
  padding: 1rem 0;
  width: 100%;
}

.ui-progress-track {
  width: 100%;
  max-width: 260px;
  height: 4px;
  background-color: var(--bg-00);
  border-radius: 2px;
  overflow: hidden;
  position: relative;
}

.ui-progress-bar {
  width: 50%;
  height: 100%;
  background: linear-gradient(90deg, var(--color-blue), var(--color-blue-hover));
  animation: global-progress-slide 1.5s infinite ease-in-out;
}

@keyframes global-progress-slide {
  0% {
    transform: translateX(-100%);
  }

  100% {
    transform: translateX(200%);
  }
}

.ui-progress-label {
  font-size: 0.85rem;
  color: var(--text-02);
}

.engine-status-box {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  user-select: none;
}

.engine-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--color-red);
}

.connected .engine-status-dot {
  background-color: var(--color-green);
}

.engine-status-txt {
  font-size: 0.85rem;
  color: var(--text-01);
}

.engine-state-highlight {
  font-weight: 600;
  color: var(--text-00);
  text-transform: capitalize;
}
</style>