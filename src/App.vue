<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue"; // Mounted and unmounted lifecycle hooks for component initialization and cleanup
import type { Tab, TabId } from './types'; // My custom types for tabs
import SvgIcon from './components/SvgIcon.vue'; // My custom component for rendering SVG icons
import { moveWindow, Position } from '@tauri-apps/plugin-positioner'; // Tauri plugin for window positioning
import { invoke } from "@tauri-apps/api/core"; // Tauri API for invoking Rust commands from the frontend
import { open } from '@tauri-apps/plugin-dialog'; // Tauri plugin for opening system dialogs

// Center the window on application launch
moveWindow(Position.Center).catch((err) => {
  console.error('Error centering window:', err);
});

// --- TAB MANAGEMENT --- //

const activeTab = ref<TabId>('ai');

const tabs: Tab[] = [
  { id: 'ai', label: 'AI Assistant', icon: 'ai' },
  { id: 'logbook', label: 'Logbook', icon: 'logbook' },
  { id: 'lifewiki', label: 'LifeWiki', icon: 'lifewiki' },
  { id: 'tasks', label: 'Tasks', icon: 'tasks' },
  { id: 'health', label: 'Mental Health', icon: 'health' },
  { id: 'settings', label: 'Settings', icon: 'settings' }
];

const selectTab = (id: TabId): void => {
  activeTab.value = id;
};

// --- OLLAMA STATUS MANAGEMENT --- //

type OllamaStatus = 'connected' | 'disconnected' | 'connecting';
const ollamaStatus = ref<OllamaStatus>('connecting');
let statusInterval: number | null = null; // Variable to hold the interval ID for periodic status checking

const installedModels = ref<string[]>([]);
const selectedModel = ref<string>('');

// Function to fetch available local models from the Ollama instance
const fetchModels = async () => {
  try {
    const models = await invoke<string[]>("get_installed_models");
    installedModels.value = models;
    if (models.length > 0 && !selectedModel.value) {
      selectedModel.value = models[0]; // Select the first available model by default
    }
  } catch (error) {
    console.error("Error fetching models:", error);
  }
};

// Function to verify Ollama's heartbeat status
const verifyOllama = async () => {
  if (ollamaStatus.value === 'connecting') return;
  try {
    const isAlive = await invoke<boolean>("check_ollama_status");
    ollamaStatus.value = isAlive ? 'connected' : 'disconnected';
  } catch {
    ollamaStatus.value = 'disconnected';
  }
};

// Watcher to automatically trigger model fetching once Ollama switches to connected state
watch(ollamaStatus, (newStatus) => {
  if (newStatus === 'connected') {
    fetchModels();
  } else if (newStatus === 'disconnected') {
    installedModels.value = [];
    selectedModel.value = '';
  }
});

// Function to handle the click event on the status indicator wrapper to boot Ollama up
const handleStatusClick = async () => {
  if (ollamaStatus.value !== 'disconnected') return;

  ollamaStatus.value = 'connecting';
  try {
    // Launch Ollama engine using our custom detached process command in Rust
    await invoke("start_ollama");

    // Loop and wait for Ollama to become ready by polling its endpoint periodically
    for (let i = 0; i < 20; i++) {
      await new Promise(resolve => setTimeout(resolve, 500));
      const isAlive = await invoke<boolean>("check_ollama_status");
      if (isAlive) {
        ollamaStatus.value = 'connected';
        return;
      }
    }
    ollamaStatus.value = 'disconnected';
  } catch (error) {
    console.error("Failed to boot Ollama process:", error);
    ollamaStatus.value = 'disconnected';
  }
};

// Lifecycle initialization hooks
onMounted(async () => {
  // Load local settings from disk immediately on app startup
  try {
    const storedSettings = await invoke<OllamaSettings>("load_ollama_settings");
    ollamaSettings.value = storedSettings;
  } catch (err) {
    console.error("Failed to load Ollama settings:", err);
  }

  invoke<boolean>("check_ollama_status")
    .then(isAlive => {
      ollamaStatus.value = isAlive ? 'connected' : 'disconnected';
      if (isAlive) fetchModels();
    })
    .catch(() => ollamaStatus.value = 'disconnected');

  statusInterval = window.setInterval(verifyOllama, 5000);
});

// Clear the interval loops on component destruction to prevent leaks
onUnmounted(() => {
  if (statusInterval) clearInterval(statusInterval);
});

// --- AI MANAGEMENT --- //

const userInput = ref<string>('');
const aiResult = ref<string>('');
const isProcessing = ref<boolean>(false);

// Function to send the current raw dump to the back-end processing pipeline
const submitBrainDump = async () => {
  if (!userInput.value.trim() || !selectedModel.value) return;

  isProcessing.value = true;
  aiResult.value = ""; // Reset preview UI state

  try {
    const result = await invoke<string>("process_brain_dump", {
      selectedModel: selectedModel.value,
      rawDump: userInput.value
    });
    aiResult.value = result;
  } catch (error) {
    aiResult.value = "Error during generation: " + error;
  } finally {
    isProcessing.value = false;
  }
};

// --- OLLAMA SETTINGS CONFIGURATION --- //

// We mirror the Rust struct structure to keep data aligned
interface OllamaSettings {
  ollama_custom_path: string;
  ollama_use_custom_path: boolean;
}

const ollamaSettings = ref<OllamaSettings>({
  ollama_custom_path: "",
  ollama_use_custom_path: false,
});

// Sends the reactive settings state down to Rust to write to settings.json
const saveSettings = async () => {
  try {
    await invoke("save_ollama_settings", { settings: ollamaSettings.value });
  } catch (err) {
    console.error("Failed to save Ollama settings:", err);
  }
};

// Opens a native system file selector strictly locked to Ollama binaries
const browseCustomPath = async () => {
  try {
    const isWindows = navigator.userAgent.includes('Windows');

    // We define strict name matching rules inside the extension filters
    const fileFilters = isWindows 
      ? [{ name: 'Ollama Executable (Ollama App.exe)', extensions: ['exe'] }]
      : [{ name: 'Ollama Binary (ollama)', extensions: [''] }]; // Unix looks for extensionless binaries

    const selected = await open({
      multiple: false,
      directory: false,
      filters: fileFilters
    });

    if (selected && typeof selected === 'string') {
      // Security guard: extract the file name from the path to double check
      const fileName = selected.split(/[/\\]/).pop()?.toLowerCase();
      
      if (isWindows && fileName !== 'ollama app.exe' && fileName !== 'ollama.exe') {
        console.warn("Selected file is not a valid Ollama executable.");
        return; // Silent block or you could hook a UI warning state here later
      } else if (!isWindows && fileName !== 'ollama') {
        console.warn("Selected file is not a valid Ollama binary.");
        return;
      }

      ollamaSettings.value.ollama_custom_path = selected;
      await saveSettings();
    }
  } catch (err) {
    console.error("Failed to open file picker:", err);
  }
};
</script>

<template>
  <div class="app-layout">
    <aside class="sidebar-left">
      <nav class="nav-links">
        <button v-for="tab in tabs" :key="tab.id" :class="['nav-btn', { active: activeTab === tab.id }]"
          @click="selectTab(tab.id)">
          <SvgIcon :name="tab.icon" class="tab-icon" />
        </button>
      </nav>
    </aside>

    <main class="main-content">
      <div v-if="activeTab === 'ai'" class="tab-panel ai-layout">
        <div class="header-zone">
          <h1>AI Assistant</h1>
          <p>Dump your thoughts, raw ideas, or your entire day. The local AI will sort everything automatically.</p>
        </div>

        <div class="dump-container">
          <div class="input-card">
            <textarea v-model="userInput"
              placeholder="Type or paste your unstructured day logs, notes, or tasks here..." class="dump-textarea"
              :disabled="isProcessing"></textarea>

            <div class="action-bar">
              <div :class="['status-container', ollamaStatus]" @click="handleStatusClick"
                :title="ollamaStatus === 'disconnected' ? 'Click to launch Ollama' : ''">
                <span v-if="ollamaStatus === 'connecting'" class="spinner"></span>
                <span v-else class="dot"></span>
                <span class="status-text">
                  Ollama Status:
                  <span class="state-label">
                    {{ ollamaStatus === 'connected' ? 'Ready' : ollamaStatus === 'connecting' ? 'Connecting...' :
                      'Disconnected (Click to run)' }}
                  </span>
                </span>
              </div>

              <div class="controls-group">
                <select v-model="selectedModel" class="model-select"
                  :disabled="ollamaStatus !== 'connected' || isProcessing">
                  <option value="" disabled>Select a model...</option>
                  <option v-for="model in installedModels" :key="model" :value="model">
                    {{ model }}
                  </option>
                </select>

                <button class="process-btn" @click="submitBrainDump"
                  :disabled="ollamaStatus !== 'connected' || !userInput.trim() || isProcessing">
                  Process Brain Dump
                </button>
              </div>
            </div>
          </div>

          <div class="preview-card">
            <h3>AI Classification Preview</h3>

            <div v-if="isProcessing" class="loading-container">
              <div class="loading-bar-wrapper">
                <div class="loading-bar-fill"></div>
              </div>
              <span class="loading-text">The model of your choice is processing your raw logs...</span>
            </div>

            <div v-else-if="aiResult" class="preview-content">
              <pre style="white-space: pre-wrap; font-family: inherit; color: #fff;">{{ aiResult }}</pre>
            </div>

            <div v-else class="preview-placeholder">
              Your processed tasks, wiki logs, and mood analysis will appear here for validation.
            </div>
          </div>
        </div>
      </div>

      <div class="tab-panel" v-else-if="activeTab === 'logbook'">
        <h1>Logbook</h1>
        <p>Your story, written in real-time.</p>
      </div>

      <div class="tab-panel" v-else-if="activeTab === 'lifewiki'">
        <h1>LifeWiki</h1>
        <p>Your personal documentation organized in a tree structure.</p>
      </div>

      <div class="tab-panel" v-else-if="activeTab === 'tasks'">
        <h1>Tasks</h1>
        <p>Your concrete action plans sorted by projects.</p>
      </div>

      <div class="tab-panel" v-else-if="activeTab === 'health'">
        <h1>Mental Health Tracker</h1>
        <p>Your mood tracker and personalized AI advice.</p>
      </div>

      <div class="tab-panel" v-else-if="activeTab === 'settings'">
        <h1>Application Settings</h1>
        <p>Configure internal components, backend runners, and paths.</p>

        <div class="settings-grid">
          <div class="settings-card">

            <!-- Modern Switch Toggle Row -->
            <div class="setting-row">
              <label class="switch">
                <input type="checkbox" v-model="ollamaSettings.ollama_use_custom_path" @change="saveSettings" />
                <span class="slider"></span>
              </label>
              <span class="setting-label">Enable Custom Executable Path Override</span>
            </div>

            <!-- Text Input combined with a Browse Button -->
            <div class="setting-item">
              <div class="input-group">
                <input type="text" v-model="ollamaSettings.ollama_custom_path"
                  placeholder="Custom Binary Path Location..." :disabled="!ollamaSettings.ollama_use_custom_path"
                  @change="saveSettings" class="settings-text-input" />
                <button @click="browseCustomPath" :disabled="!ollamaSettings.ollama_use_custom_path" class="browse-btn">
                  Browse...
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
html,
body {
  margin: 0;
  padding: 0;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  background-color: #1a1a1a;
  color: #fff;
}

.app-layout {
  background-color: #1a1a1a;
  display: flex;
  width: 100%;
  height: 100vh;
}

.sidebar-left {
  background-color: #242424;
  border-right: 1px solid #333;
  padding: 1.5rem 1rem;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.nav-links {
  align-self: left;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.nav-btn {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  background: none;
  width: content;
  border: none;
  color: #a0a0a0;
  padding: 1rem;
  text-align: left;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.95rem;
  transition: all 0.2s ease;
}

.nav-btn:hover {
  background-color: #2e2e2e;
  box-shadow: inset 0 0 0 2px rgba(129, 129, 129, 0.5);
  color: #fff;
}

.nav-btn.active {
  background-color: #242424;
  box-shadow: inset 0 0 0 2px #3b82f6;
  color: #fff;
  font-weight: 500;
}

.tab-icon {
  font-size: 1.1rem;
}

.main-content {
  flex: 1;
  background-color: #1e1e1e;
  padding: 2.5rem;
  overflow-y: auto;
}

.tab-panel h1 {
  margin-top: 0;
  font-size: 1.8rem;
  color: #fff;
}

.tab-panel p {
  color: #a0a0a0;
  font-size: 1rem;
}

.ai-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 1.5rem;
}

.header-zone {
  flex-shrink: 0;
}

.dump-container {
  display: flex;
  gap: 1.5rem;
  flex: 1;
  min-height: 0;
}

.input-card,
.preview-card {
  flex: 1;
  background-color: #242424;
  border: 1px solid #333;
  border-radius: 8px;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.preview-card h3 {
  margin-top: 0;
  color: #fff;
  font-size: 1.1rem;
}

.dump-textarea {
  flex: 1;
  background-color: #1a1a1a;
  border: 1px solid #333;
  color: #fff;
  border-radius: 6px;
  padding: 1rem;
  resize: none;
  font-family: inherit;
  font-size: 0.95rem;
  line-height: 1.5;
}

.dump-textarea:focus {
  outline: none;
  border-color: #3b82f6;
}

.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 1rem;
  flex-shrink: 0;
}

.status-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  user-select: none;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.status-container.disconnected {
  cursor: pointer;
  color: #ef4444;
}

.status-container.disconnected:hover {
  background-color: rgba(239, 68, 68, 0.1);
}

.status-container.disconnected .dot {
  background-color: #ef4444;
}

.status-container.connected {
  color: #4ade80;
}

.status-container.connected .dot {
  background-color: #4ade80;
}

.status-container.connecting {
  color: #3b82f6;
}

.status-container .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}

.status-container .state-label {
  font-weight: 600;
}

.status-container .spinner {
  width: 10px;
  height: 10px;
  border: 2px solid rgba(59, 130, 246, 0.3);
  border-radius: 50%;
  border-top-color: #3b82f6;
  animation: spin 0.8s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.process-btn {
  background-color: #3b82f6;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s;
}

.process-btn:hover {
  background-color: #2563eb;
}

.process-btn:disabled {
  background-color: #444;
  color: #888;
  cursor: not-allowed;
}

.preview-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: #666;
  border: 2px dashed #333;
  border-radius: 6px;
  padding: 2rem;
  font-size: 0.95rem;
}

.controls-group {
  display: flex;
  gap: 0.75rem;
  align-items: center;
}

.model-select {
  background-color: #1a1a1a;
  border: 1px solid #333;
  color: #fff;
  padding: 0.75rem 1rem;
  border-radius: 6px;
  font-family: inherit;
  font-size: 0.95rem;
  cursor: pointer;
  outline: none;
  transition: border-color 0.2s;
}

.model-select:focus:not(:disabled) {
  border-color: #3b82f6;
}

.model-select:disabled {
  background-color: #2a2a2a;
  color: #666;
  cursor: not-allowed;
  border-color: #222;
}

.loading-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 1rem;
}

.loading-bar-wrapper {
  width: 60%;
  height: 4px;
  background-color: #1a1a1a;
  border-radius: 2px;
  overflow: hidden;
  position: relative;
}

.loading-bar-fill {
  width: 0%;
  height: 100%;
  background-color: #3b82f6;
  border-radius: 2px;
  position: absolute;
  animation: loading-slide 1.5s infinite linear;
}

.loading-text {
  font-size: 0.9rem;
  color: #888;
}

@keyframes loading-slide {
  0% {
    left: 0%;
    width: 0%;
  }

  16% {
    left: 0%;
    width: 50%;
  }

  50% {
    left: 25%;
    width: 50%;
  }

  83% {
    left: 50%;
    width: 50%;
  }

  100% {
    left: 100%;
    width: 0%;
  }
}

/* --- SETTINGS TAB STYLES --- */
.settings-grid {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  margin-top: 2rem;
}

.settings-card {
  background-color: #242424;
  border: 1px solid #333;
  border-radius: 8px;
  padding: 1.5rem;
}

.setting-item {
  margin-bottom: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.setting-item:last-child {
  margin-bottom: 0;
}

.check-toggle label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  user-select: none;
  font-size: 0.95rem;
  color: #fff;
}

.setting-item label {
  font-size: 0.9rem;
  font-weight: 500;
  color: #fff;
}
/* Layout helper for the toggle row */
.setting-row {
  display: flex;
  align-items: center;
  margin-bottom: 1.5rem;
}

.setting-label {
  font-size: 0.95rem;
  color: #fff;
  font-weight: 500;
}

/* --- THE TOGGLE SWITCH --- */
.switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  margin-right: 1rem;
}

.switch input { 
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: #333;
  transition: .3s;
  border-radius: 24px;
  border: 1px solid #444;
}

.slider:before {
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

input:checked + .slider {
  background-color: #3b82f6;
  border-color: #2563eb;
}

input:checked + .slider:before {
  transform: translateX(20px);
  background-color: #fff;
}

input:disabled + .slider {
  opacity: 0.5;
  cursor: not-allowed;
}

/* --- INPUT GROUP WITH BROWSE BUTTON --- */
.input-group {
  display: flex;
  gap: 0.5rem;
  max-width: 500px;
}

.input-group .settings-text-input {
  flex: 1; /* Input takes all remaining space */
  max-width: none; 
  background-color: #2e2e2e;
  border: none;
  color: #fff;
  padding-left: 0.75rem;
  border-radius: 12px;
}

.input-group .settings-text-input:disabled {
  background-color: #2e2e2e;
  color: #666; /* Muted text when disabled */
  cursor: not-allowed;
  border-color: #222;
}

.browse-btn {
  background-color: #333;
  color: #fff;
  border: 1px solid #444;
  padding: 0.6rem 1rem;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 500;
  transition: all 0.2s;
}

.browse-btn:hover:not(:disabled) {
  background-color: #444;
  border-color: #555;
}

.browse-btn:disabled {
  background-color: #111;
  color: #555;
  border-color: #222;
  cursor: not-allowed;
}
</style>