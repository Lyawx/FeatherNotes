<script setup lang="ts">
import { ref, onMounted } from "vue";
import { mainService } from "../services/mainService";
import type { FileNode } from "../services/logbookService";

const mdFiles = ref<FileNode[]>([]);
const activeFile = ref<FileNode | null>(null);
const renderedHtml = ref<string>("");
const isLoading = ref<boolean>(true);

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
        <p>Aucun fichier .md dans le dossier Logbook.</p>
        <span class="hint">Ajoutez des fichiers Markdown pour les voir apparaître ici.</span>
      </div>

      <ul v-else class="file-list">
        <li v-for="file in mdFiles" :key="file.path" :class="['file-item', { active: activeFile?.path === file.path }]"
          @click="selectFile(file)">
          {{ file.name.replace(/\.md$/, '') }}
        </li>
      </ul>
    </aside>

    <main class="file-content">
      <div v-if="activeFile" class="markdown-body" v-html="renderedHtml"></div>
      <div v-else-if="!isLoading" class="preview-placeholder">
        Sélectionnez une note dans la barre latérale pour l'afficher.
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
  background-color: #1a1a1a;
  border-right: 1px solid #2a2a2a;
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
  color: #9ca3af;
  margin-bottom: 0.25rem;
}

.empty-state .hint {
  font-size: 0.8rem;
  color: #6b7280;
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
  color: #9ca3af;
  font-size: 0.95rem;
  transition: all 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-item:hover {
  background-color: #242424;
  color: #fff;
}

.file-item.active {
  background-color: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
  font-weight: 500;
}

.file-content {
  flex: 1;
  padding: 2.5rem;
  overflow-y: auto;
  background-color: #141414;
}

.preview-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #4b5563;
  font-size: 1.1rem;
}
</style>