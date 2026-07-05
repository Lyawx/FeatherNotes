<script setup lang="ts">
import { ref, onMounted } from "vue";
import { open } from '@tauri-apps/plugin-dialog';
import { mainService } from "../services/mainService";
import type { AppSettings } from "../services/settingsService";
import type { OllamaStatus } from "../services/ollamaService";

const props = defineProps<{ ollamaStatus: string }>();
const emit = defineEmits<{ (e: 'updateStatus', status: OllamaStatus): void }>();

const appSettings = ref<AppSettings>({
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

onMounted(async () => {
  try { appSettings.value = await mainService.settings.load(); } catch (err) { console.error(err); }
});

const saveSettings = async () => {
  try { await mainService.settings.save(appSettings.value); } catch (err) { console.error(err); }
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
    const isDirectory = target === 'markdown'; // true pour dossier (markdown), false pour fichier (ollama)

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
    <p>Configure internal components, backend runners, and paths.</p>

    <div class="container-light">
      <h2>Ollama</h2>
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
        <button id="start-ollama-btn" class="base-btn" @click="startOllama"
          :disabled="ollamaStatus === 'connected' || ollamaStatus === 'connecting'">
          <span v-if="ollamaStatus === 'connecting'">Starting Ollama...</span>
          <span v-else>Start Ollama Process</span>
        </button>

        <button id="stop-ollama-btn" class="base-btn" @click="stopOllama" :disabled="ollamaStatus !== 'connected'">
          Force Stop Ollama Process
        </button>
      </div>
    </div>

    <div class="container-light">
      <h2>Storage</h2>
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
        <button id="browse-md-btn" class="base-btn" @click="browsePath('markdown')"
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
}

.setting-label {
  font-size: 0.95rem;
  font-weight: 500;
}

.input-group {
  display: flex;
  gap: 0.5rem;
}

#settings-action-block {
  display: flex;
  gap: 1rem;
}

#browse-path-btn {
  background-color: rgba(255, 255, 255, 0.08);
  color: #fff;
  border-color: rgba(255, 255, 255, 0.1);
}

#browse-path-btn:hover:not(:disabled) {
  background-color: rgba(255, 255, 255, 0.15);
}

#start-ollama-btn {
  background-color: rgba(74, 222, 128, 0.1);
  color: #4ade80;
  border-color: rgba(74, 222, 128, 0.3);
}

#start-ollama-btn:hover:not(:disabled) {
  background-color: #4ade80;
  color: #1a1a1a;
  box-shadow: 0 0 12px rgba(74, 222, 128, 0.2);
}

#stop-ollama-btn {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

#stop-ollama-btn:hover:not(:disabled) {
  background-color: #ef4444;
  color: #fff;
  box-shadow: 0 0 12px rgba(239, 68, 68, 0.2);
}
</style>