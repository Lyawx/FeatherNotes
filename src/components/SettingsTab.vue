<script setup lang="ts">
import { ref, onMounted } from "vue";
import { open } from '@tauri-apps/plugin-dialog';
import { mainService } from "../services/mainService";
import type { AppSettings } from "../services/settingsService";
import type { OllamaStatus } from "../services/ollamaService";

const props = defineProps<{ ollamaStatus: string }>();
const emit = defineEmits<{ (e: 'updateStatus', status: OllamaStatus): void }>();

const appSettings = ref<AppSettings>({
  urgent_threshold_days: 1,
  warning_threshold_days: 3,
  theme: "default",
  ollama: {
    ollama_default_path: "",
    ollama_custom_path: "",
    ollama_use_custom_path: false
  },
  markdown: {
    markdown_default_path: "",
    markdown_custom_path: "",
    markdown_use_custom_path: false
  }
});

const availableThemes = ref<string[]>([]);

onMounted(async () => {
  try { 
    appSettings.value = await mainService.settings.loadSettings(); 
    availableThemes.value = await mainService.themes.getAvailableThemes();
  } catch (err) { 
    console.error(err); 
  }
});

const saveSettings = async () => {
  try { 
    await mainService.settings.saveSettings(appSettings.value); 
  } catch (err) { 
    console.error(err); 
  }
};

const handleThemeChange = async () => {
  try {
    await saveSettings();
    await mainService.themes.applyTheme(appSettings.value.theme);
  } catch (err) {
    console.error(err);
  }
};

const startOllama = async () => {
  if (props.ollamaStatus !== 'disconnected') return;
  emit('updateStatus', 'connecting');
  try {
    await mainService.ollama.startProcess();
    for (let i = 0; i < 20; i++) {
      await new Promise(resolve => setTimeout(resolve, 500));
      if (await mainService.ollama.checkStatus()) {
        emit('updateStatus', 'connected');
        return;
      }
    }
    emit('updateStatus', 'disconnected');
  } catch { emit('updateStatus', 'disconnected'); }
};

const stopOllama = async () => {
  try {
    await mainService.ollama.stopProcess();
    emit('updateStatus', 'disconnected');
  } catch (err) { console.error(err); }
};

const browsePath = async (target: 'ollama' | 'markdown') => {
  try {
    const isDirectory = target === 'markdown';

    const selected = await open({
      multiple: false,
      directory: isDirectory
    });

    if (selected && typeof selected === 'string') {
      if (target === 'ollama') {
        appSettings.value.ollama.ollama_custom_path = selected;
      } else {
        appSettings.value.markdown.markdown_custom_path = selected;
      }
      await saveSettings();
    }
  } catch (err) {
    console.error(`Failed to browse path for ${target}:`, err);
  }
};
</script>

<template>
  <div id="settings-panel" class="tab-panel">
    <h1>Application Settings</h1>
    <p class="tab-desc">Configure internal components, backend runners, and paths.</p>

    <div class="container-light">
      <h3>Interface Theme</h3>
      <hr class="ui-divider" />
      <div class="setting-row">
        <select 
          v-model="appSettings.theme" 
          @change="handleThemeChange" 
          class="text-input-field select-dropdown"
        >
          <option v-for="theme in availableThemes" :key="theme" :value="theme">
            {{ theme }}
          </option>
        </select>
      </div>
    </div>

    <div class="container-light">
      <h3>Task Deadlines</h3>
      <hr class="ui-divider" />
      
      <div class="numeric-settings-group">
        <div class="setting-row-numeric">
          <span class="setting-label">Urgent Threshold (Days)</span>
          <input 
            type="number" 
            v-model.number="appSettings.urgent_threshold_days" 
            @change="saveSettings" 
            class="text-input-field numeric-input"
            min="0"
          />
        </div>

        <div class="setting-row-numeric">
          <span class="setting-label">Warning Threshold (Days)</span>
          <input 
            type="number" 
            v-model.number="appSettings.warning_threshold_days" 
            @change="saveSettings" 
            class="text-input-field numeric-input"
            min="0"
          />
        </div>
      </div>
    </div>

    <div class="container-light">
      <h3>Ollama</h3>
      <hr class="ui-divider" />
      <div class="setting-row">
        <label class="ui-toggle-wrapper">
          <input type="checkbox" v-model="appSettings.ollama.ollama_use_custom_path" @change="saveSettings" />
          <span class="ui-toggle-slider"></span>
        </label>
        <span class="setting-label">Enable Custom Executable Path Override</span>
      </div>

      <div class="input-group">
        <input type="text" v-model="appSettings.ollama.ollama_custom_path"
          :placeholder="appSettings.ollama.ollama_default_path || 'Recherche du chemin par défaut...'"
          :disabled="!appSettings.ollama.ollama_use_custom_path" @change="saveSettings" class="text-input-field" />
        <button id="browse-path-btn" class="base-btn" @click="browsePath('ollama')"
          :disabled="!appSettings.ollama.ollama_use_custom_path">
          Browse...
        </button>
      </div>

      <div id="settings-action-block">
        <button class="base-btn green-btn" @click="startOllama"
          :disabled="ollamaStatus === 'connected' || ollamaStatus === 'connecting'">
          <span v-if="ollamaStatus === 'connecting'">Starting Ollama...</span>
          <span v-else>Start Ollama Process</span>
        </button>

        <button class="base-btn red-btn" @click="stopOllama" :disabled="ollamaStatus !== 'connected'">
          Force Stop Ollama Process
        </button>
      </div>
    </div>

    <div class="container-light">
      <h3>Storage</h3>
      <hr class="ui-divider" />
      <div class="setting-row">
        <label class="ui-toggle-wrapper">
          <input type="checkbox" v-model="appSettings.markdown.markdown_use_custom_path" @change="saveSettings" />
          <span class="ui-toggle-slider"></span>
        </label>
        <span class="setting-label">Use Custom Directory Path for Vault</span>
      </div>

      <div class="input-group">
        <input type="text" v-model="appSettings.markdown.markdown_custom_path"
          :placeholder="appSettings.markdown.markdown_default_path || 'Recherche du dossier par défaut...'"
          :disabled="!appSettings.markdown.markdown_use_custom_path" @change="saveSettings" class="text-input-field" />
        <button class="base-btn dark-btn" @click="browsePath('markdown')"
          :disabled="!appSettings.markdown.markdown_use_custom_path">
          Browse...
        </button>
      </div>
    </div>

  </div>
</template>

<style scoped>
.setting-row {
  display: flex;
  align-items: center;
  margin-bottom: 1rem;
}

.numeric-settings-group {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-width: 300px;
}

.setting-row-numeric {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.numeric-input {
  width: 70px;
  text-align: center;
}

.setting-label {
  font-size: 0.95rem;
  font-weight: 500;
}

.setting-description {
  font-size: 0.85rem;
  color: var(--text-02);
  margin-bottom: 1rem;
}

.input-group {
  display: flex;
  gap: 0.5rem;
}

#settings-action-block {
  display: flex;
  gap: 1rem;
}

.select-dropdown {
  width: 100%;
  max-width: 300px;
  cursor: pointer;
}
</style>