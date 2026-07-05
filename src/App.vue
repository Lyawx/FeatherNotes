<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import SvgIcon from './components/SvgIcon.vue';
import AiTab from './components/AiTab.vue';
import SettingsTab from './components/SettingsTab.vue';
import LogbookTab from "./components/LogbookTab.vue";
import { mainService, type OllamaStatus } from "./services/mainService";

interface Tab {
  id: 'ai' | 'logbook' | 'lifewiki' | 'tasks' | 'health' | 'settings';
  label: string;
  icon: string;
}

const activeTab = ref<Tab['id']>('ai');

// Standard main navigation tabs
const mainTabs: Tab[] = [
  { id: 'ai', label: 'AI Assistant', icon: 'ai' },
  { id: 'logbook', label: 'Logbook', icon: 'logbook' },
  { id: 'lifewiki', label: 'LifeWiki', icon: 'lifewiki' },
  { id: 'tasks', label: 'Tasks', icon: 'tasks' },
  { id: 'health', label: 'Mental Health', icon: 'health' }
];

// Isolated settings tab for layout positioning
const settingsTab: Tab = { id: 'settings', label: 'Settings', icon: 'settings' };

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
    <aside class="sidebar-left">
      <nav class="nav-links">
        <div class="main-nav-group">
          <button v-for="tab in mainTabs" :key="tab.id" :class="['nav-btn', { active: activeTab === tab.id }]"
            @click="activeTab = tab.id">
            <SvgIcon :name="tab.icon" class="tab-icon" />
          </button>
        </div>

        <button :class="['nav-btn', 'settings-nav-btn', { active: activeTab === settingsTab.id }]"
          @click="activeTab = settingsTab.id">
          <SvgIcon :name="settingsTab.icon" class="tab-icon" />
        </button>
      </nav>
    </aside>

    <main class="main-content">
      <AiTab v-if="activeTab === 'ai'" :ollamaStatus="ollamaStatus" @triggerStart="handleForcedStart" />

      <SettingsTab v-else-if="activeTab === 'settings'" :ollamaStatus="ollamaStatus" @updateStatus="ollamaStatus = $event" />

      <LogbookTab v-else-if="activeTab === 'logbook'" />

      <div class="tab-panel" v-else>
        <h1>{{mainTabs.find(t => t.id === activeTab)?.label}}</h1>
        <div class="container-light">
          <p>This is a light container block.</p>
          <div class="container-dark">
            <p>This is an embedded dark container block.</p>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
/* --- GLOBAL ATOMIC DESIGN SYSTEM --- */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  font-family: Inter, system-ui, sans-serif;
  color-scheme: dark;
  color: rgba(255, 255, 255, 0.9);
}

body,
html {
  width: 100vw;
  height: 100vh;
  background-color: #141414;
  overflow: hidden;
}

/* --- FRAMEWORK WORKSPACE LAYOUTS --- */
.app-layout {
  display: flex;
  width: 100vw;
  height: 100vh;
}

.sidebar-left {
  width: 70px;
  background-color: #1a1a1a;
  border-right: 1px solid #2a2a2a;
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

/* Pushes the settings button to the absolute floor of the flexbox grid */
.settings-nav-btn {
  margin-top: auto;
}

.main-content {
  flex: 1;
  height: 100vh;
  overflow-y: auto;
  background-color: #141414;
}

/* --- NAVIGATION BUTTONS --- */
.nav-btn {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  border: 1px solid transparent;
  background-color: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  color: #666;
}

.nav-btn:hover {
  background-color: #242424;
  color: #fff;
}

.nav-btn.active {
  background-color: #3b82f6;
  color: #fff;
  border-color: #2563eb;
}

.tab-icon {
  width: 22px;
  height: 22px;
  fill: currentColor;
}

/* --- TYPOGRAPHY --- */
.tab-panel {
  padding: 2.5rem;
  min-height: 100%;
}

h1 {
  font-size: 2rem;
  font-weight: 700;
  color: #fff;
  margin-bottom: 0.5rem;
  letter-spacing: -0.025em;
}

h2 {
  font-size: 1.5rem;
  font-weight: 500;
  color: #fff;
  margin-bottom: 1rem;
}

h3 {
  font-size: 1.2rem;
  font-weight: 500;
  color: #f3f4f6;
  margin-bottom: 1rem;
}

p {
  font-size: 0.95rem;
  color: #9ca3af;
  margin-bottom: 1.5rem;
}

/* --- TWO CONCRETE STANDALONE CONTAINERS --- */
.container-light {
  background-color: #1a1a1a;
  border: 1px solid #333333;
  border-radius: 8px;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  margin-bottom: 1.25rem;
}

.container-dark {
  background-color: #141414;
  border: 1px solid #333333;
  border-radius: 8px;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  margin-bottom: 1.25rem;
}

/* --- REUSABLE DIVIDER / SEPARATOR --- */
.ui-divider {
  height: 1px;
  width: 100%;
  background-color: rgba(255, 255, 255, 0.1);
  border: none;
}

/* --- REUSABLE INPUT FIELDS --- */
.text-input-field {
  width: 100%;
  background-color: rgba(0, 0, 0, 0.2);
  border: 1px solid #333333;
  color: #fff;
  padding: 0.65rem 0.85rem;
  border-radius: 8px;
  font-family: inherit;
  font-size: 0.95rem;
  transition: all 0.2s;
}

.text-input-field:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.text-input-field:disabled {
  background-color: rgba(255, 255, 255, 0.02);
  color: #4b5563;
  cursor: not-allowed;
  border-color: rgba(255, 255, 255, 0.05);
}

/* --- ATOMIC BUTTONS SYSTEM --- */
.base-btn {
  padding: 0.6rem 1.2rem;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 600;
  transition: all 0.2s;
  border: 1px solid transparent;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.base-btn:disabled {
  background-color: rgba(255, 255, 255, 0.02) !important;
  color: #4b5563 !important;
  border-color: rgba(255, 255, 255, 0.05) !important;
  box-shadow: none !important;
  cursor: not-allowed;
}

/* --- REUSABLE UI ANIMATIONS & SWITCHES --- */
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
  background-color: rgba(0, 0, 0, 0.3);
  transition: .3s;
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.ui-toggle-slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: #aaa;
  transition: .3s;
  border-radius: 50%;
}

input:checked+.ui-toggle-slider {
  background-color: #3b82f6;
  border-color: #2563eb;
}

input:checked+.ui-toggle-slider:before {
  transform: translateX(20px);
  background-color: #fff;
}

.ui-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid #aaa;
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
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
  overflow: hidden;
  position: relative;
}

.ui-progress-bar {
  width: 50%;
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #60a5fa);
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
  color: #6b7280;
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
  background-color: #ef4444;
}

.connected .engine-status-dot {
  background-color: #4ade80;
}

.engine-status-txt {
  font-size: 0.85rem;
  color: #9ca3af;
}

.engine-state-highlight {
  font-weight: 600;
  color: #fff;
  text-transform: capitalize;
}

/* --- MARLDOWN STYLE --- */
.markdown-body :deep(h1) {
  font-size: 1.8rem;
  margin-bottom: 1rem;
  color: #fff;
}

.markdown-body :deep(h2) {
  font-size: 1.4rem;
  margin-top: 1.5rem;
  margin-bottom: 0.75rem;
  color: #fff;
}

.markdown-body :deep(p) {
  margin-bottom: 1rem;
  line-height: 1.6;
  color: #d1d5db;
}

.markdown-body :deep(ul) {
  margin-left: 1.5rem;
  margin-bottom: 1rem;
  color: #d1d5db;
}

.markdown-body :deep(li) {
  margin-bottom: 0.25rem;
}
</style>