<script setup lang="ts">
import { ref, onMounted } from "vue";
import { mainService } from "../services/mainService";
import type { FileNode } from "../services/logbookService";
import MarkdownEditor from "./MarkdownEditor.vue";

const mdFiles = ref<FileNode[]>([]);
const activeFile = ref<FileNode | null>(null);
const renderedHtml = ref<string>("");
const isLoading = ref<boolean>(true);
const rawText = ref<string>("");
const isEditing = ref<boolean>(false);

const loadLogbook = async () => {
  isLoading.value = true;
  try {
    mdFiles.value = await mainService.logbook.getLogbookFiles();
    // Optionnel : Sélectionne automatiquement le premier fichier s'il y en a un
    if (mdFiles.value.length > 0) {
      selectFile(mdFiles.value[0]);
    }
  } catch (err) {
    console.error("Erreur d'initialisation du Logbook :", err);
  } finally {
    isLoading.value = false;
  }
};

const selectFile = async (file: FileNode) => {
  activeFile.value = file;
  renderedHtml.value = "Loading content...";
  try {
    renderedHtml.value = await mainService.logbook.getFileHtml(file.path);
  } catch (err) {
    renderedHtml.value = `<p style="color: #ef4444;">Erreur de lecture du fichier : ${err}</p>`;
  }
};

const startEditing = async () => {
  if (!activeFile.value) return;
  try {
    rawText.value = await mainService.logbook.getFileRawText(activeFile.value.path);
    isEditing.value = true;
  } catch (err) {
    alert("Impossible de charger le texte brut : " + err);
  }
};

const saveChanges = async () => {
  if (!activeFile.value) return;
  try {
    await mainService.logbook.saveFileRawText(activeFile.value.path, rawText.value);
    // On met à jour le HTML affiché
    renderedHtml.value = await mainService.logbook.getFileHtml(activeFile.value.path);
    isEditing.value = false;
  } catch (err) {
    alert("Erreur lors de la sauvegarde : " + err);
  }
};

onMounted(() => {
  loadLogbook();
});
</script>

<template>
  <div id="logbook-panel">
    <aside class="sidebar-files">
      <h3>Journal Entries</h3>
      <hr class="ui-divider" style="margin: 0.75rem 0;" />

      <div v-if="isLoading" class="ui-spinner-container">
        <span class="ui-spinner"></span>
      </div>

      <div v-else-if="mdFiles.length === 0" class="empty-state">
        <p>No markdown files found in the Logbook folder.</p>
        <span class="hint">Add markdown files to see them displayed here.</span>
      </div>

      <ul v-else class="file-list">
        <li v-for="file in mdFiles" :key="file.path" :class="['file-item', { active: activeFile?.path === file.path }]"
          @click="selectFile(file)">
          {{ file.name.replace(/\.md$/, '') }}
        </li>
      </ul>
    </aside>

    <main class="file-content">
      <div v-if="activeFile" class="content-wrapper">

        <div class="note-actions">
          <span class="file-title">{{ activeFile.name }}</span>
          <button v-if="!isEditing" @click="startEditing" class="base-btn">
            <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
              <title xmlns="">pen</title>
              <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
            </svg>
          </button>
          <div v-else class="edit-buttons-group">
            <button @click="saveChanges" class="base-btn green-btn">
              <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                <title xmlns="">save-line</title>
                <path fill="currentColor"
                  d="M7 19v-6h10v6h2V7.828L16.172 5H5v14zM4 3h13l4 4v13a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1m5 12v4h6v-4z" />
              </svg>
            </button>
            <button @click="isEditing = false" class="base-btn red-btn">Annuler</button>
          </div>
        </div>

        <hr class="ui-divider" style="margin-bottom: 1.5rem;" />

        <MarkdownEditor v-if="isEditing" v-model="rawText" />

        <div v-else class="markdown-body" v-html="renderedHtml"></div>

      </div>

      <div v-else-if="!isLoading" class="preview-placeholder">
        Select a file in the sidebar file explorer to display it here.
      </div>
    </main>
  </div>
</template>

<style scoped>

#logbook-panel {
  display: flex;
  height: 100%;
}

.sidebar-files {
  width: 260px;
  background-color: var(--bg-01);
  border-right: var(--border-width) solid var(--bg-02);
  padding: 1.5rem 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.ui-spinner-container {
  display: flex;
  justify-content: center;
  padding: 2rem 0;
}

.empty-state {
  padding: 1rem 0;
  text-align: center;
}

.empty-state p {
  font-size: 0.9rem;
  color: var(--text-01);
  margin-bottom: 0.25rem;
}

.empty-state .hint {
  font-size: 0.8rem;
  color: var(--text-02);
}

.file-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.file-item {
  padding: 0.65rem 0.85rem;
  cursor: pointer;
  border-radius: 8px;
  color: var(--text-01);
  font-size: 0.95rem;
  transition: all 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-item:hover {
  background-color: var(--bg-02);
  color: var(--text-00);
}

.file-item.active {
  background-color: var(--bg-active);
  color: var(--color-blue);
  font-weight: 500;
}

.file-content {
  flex: 1;
  padding: 2.5rem;
  overflow-y: auto;
  background-color: var(--bg-00);
}

.preview-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-02);
  font-size: 1.1rem;
}

.content-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.note-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 0.5rem;
  height: 64px;
}

.file-title {
  font-weight: 600;
  font-size: 1.1rem;
  color: var(--text-00);
}
</style>