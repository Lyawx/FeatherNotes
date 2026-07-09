<script setup lang="ts">
import { ref, onMounted } from "vue";
import { mainService } from "../services/mainService";
import type { FileNode } from "../services/mainService";
import MarkdownEditor from "./MarkdownEditor.vue";
import SidebarFiles from "./SidebarFiles.vue";

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
    renderedHtml.value = await mainService.item.getFileHtml(file.path);
  } catch (err) {
    renderedHtml.value = `<p style="color: #ef4444;">Erreur de lecture du fichier : ${err}</p>`;
  }
};

const startEditing = async () => {
  if (!activeFile.value) return;
  try {
    rawText.value = await mainService.item.getFileRawText(activeFile.value.path);
    isEditing.value = true;
  } catch (err) {
    alert("Impossible de charger le texte brut : " + err);
  }
};

const saveChanges = async () => {
  if (!activeFile.value) return;
  try {
    await mainService.item.saveFileRawText(activeFile.value.path, rawText.value);
    renderedHtml.value = await mainService.item.getFileHtml(activeFile.value.path);
    isEditing.value = false;
  } catch (err) {
    alert("Erreur lors de la sauvegarde : " + err);
  }
};

const createNewFile = async () => {
  const fileName = prompt("File name :");
  if (!fileName) return;
  try {
    const dirPath = await mainService.logbook.getLogbookPath();
    const separator = dirPath.includes("/") ? "/" : "\\";
    let fullPath = `${dirPath}${separator}${fileName}`;
    if (!fileName.endsWith('.md')) {
      fullPath += '.md';
    }
    await mainService.item.saveFileRawText(fullPath, "# " + fileName + "\n\nNouveau logbook.");
    await loadLogbook();
    alert("Fichier créé avec succès !");
  } catch (err) {
    console.error("Erreur :", err);
    alert("Erreur lors de la création : " + err);
  }
};

const rename = async ({ item, newName }: { item: any, newName: string }) => {
  try {
    if (!newName.trim()) return;
    if (!newName.endsWith('.md')) {
      newName += '.md';
    }
    await mainService.item.renameItem(item.path, newName);
    await loadLogbook();
    
    if (activeFile.value && activeFile.value.path === item.path) {
      activeFile.value.name = newName;
      activeFile.value.path = item.path.substring(0, item.path.lastIndexOf(item.path.includes('/') ? '/' : '\\') + 1) + newName;
    }
  } catch (error) {
    console.error("Erreur lors du renommage :", error);
  }
};

onMounted(() => {
  loadLogbook();
});
</script>

<template>
  <div id="logbook-panel">
    <SidebarFiles 
      @create-click="createNewFile" 
      @rename="rename" 
      title="Journal Entries" 
      :items="mdFiles" 
      :isLoading="isLoading"
      emptyMessage="No markdown files found in the Logbook folder."
      emptyHint="Add markdown files to see them displayed here."
      :class="{ 'has-active-child': activeFile }"
    >
      <template #item="{ item: file }">
        <div 
          :class="['file-item-trigger', { active: activeFile?.path === file.path }]" 
          @click="selectFile(file)"
        >
          {{ file.name.replace(/\.md$/, '') }}
        </div>
      </template>
    </SidebarFiles>

    <main class="file-content">
      <div v-if="activeFile" class="content-wrapper">
        
        <div v-if="!isEditing" class="note-actions">
          <span class="file-title">{{ activeFile.name }}</span>
          <button @click="startEditing" class="base-btn">
            <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
              <title>pen</title>
              <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
            </svg>
          </button>
        </div>

        <hr v-if="!isEditing" class="ui-divider" style="margin-bottom: 1.5rem;" />
        
        <MarkdownEditor
          v-if="isEditing"
          v-model="rawText"
          v-model:fileName="activeFile.name"
          v-model:filePath="activeFile.path"
          :enforce-md="true"
          @save="saveChanges"
          @cancel="isEditing = false"
        />
        <div v-else class="markdown-body" v-html="renderedHtml"></div>
      </div>

      <div v-if="!activeFile && !isLoading" class="preview-placeholder">
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

:deep(.item-row) {
  padding: 0;
  border-radius: 8px;
  color: var(--text-01);
  font-size: 0.95rem;
  transition: all 0.2s;
  cursor: pointer;
  display: flex;
}


:deep(.item-row:hover) {
  background-color: var(--bg-02);
  color: var(--text-00);
}

:deep(.item-row:has(.active)) {
  background-color: var(--bg-active);
  color: var(--color-blue);
  font-weight: 500;
}

.file-item-trigger {
  width: 100%;
  padding: 0.65rem 0.85rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
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