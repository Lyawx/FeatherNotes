<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { mainService } from "../services/mainService";

const props = withDefaults(defineProps<{
  modelValue: string;
  filePath: string;
  fileName: string;
  placeholder?: string;
  enforceMd?: boolean;
}>(), {
  enforceMd: false
});

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "update:fileName", value: string): void;
  (e: "update:filePath", newPath: string): void;
  (e: "save"): void;
  (e: "cancel"): void;
}>();

const textContent = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const currentName = ref(props.fileName);

watch(() => props.fileName, (newName) => {
  currentName.value = newName;
});

const handleRename = async () => {
  let cleanedName = currentName.value.trim();
  
  if (props.enforceMd && cleanedName && !cleanedName.toLowerCase().endsWith(".md")) {
    cleanedName += ".md";
  }

  if (!cleanedName || cleanedName === props.fileName) {
    currentName.value = props.fileName;
    return;
  }

  try {
    const result = await mainService.item.renameItem(props.filePath, cleanedName) as any;
    emit("update:fileName", cleanedName);
    
    if (result && typeof result === 'object' && 'new_path' in result) {
      emit("update:filePath", result.new_path);
    }
  } catch (error) {
    console.error("Error renaming file from editor header:", error);
    currentName.value = props.fileName;
  }
};

const defaultPlaceholder = `# Header 1\n\nWrite your markdown here...\n\n- [ ] Todo item\n- [x] Done item`;
</script>

<template>
  <div class="editor-wrapper">
    <div class="editor-header">
      <input 
        type="text" 
        v-model="currentName" 
        class="editor-textarea"
        placeholder="Untitled File"
        @blur="handleRename"
        @keydown.enter="handleRename"
      />
      
      <div class="edit-buttons-group">
        <button @click="emit('save')" class="edit-buttons base-btn green-btn">
          <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <title>save-line</title>
            <path fill="currentColor" d="M7 19v-6h10v6h2V7.828L16.172 5H5v14zM4 3h13l4 4v13a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1m5 12v4h6v-4z" />
          </svg>
        </button>
        <button @click="emit('cancel')" class="edit-buttons base-btn red-btn">
          X
        </button>
      </div>
    </div>

    <div class="editor-container">
      <textarea 
        v-model="textContent" 
        class="text-input-field editor-textarea"
        :placeholder="placeholder || defaultPlaceholder"
      ></textarea>

      <div class="editor-cheatsheet">
        <div class="cheatsheet-group">
          <span class="group-title">Headers</span>
          <span><b>#</b> H1</span>
          <span><b>##</b> H2</span>
          <span><b>###</b> H3</span>
        </div>
        <div class="cheatsheet-group">
          <span class="group-title">Inline</span>
          <span><b>**</b>Bold<b>**</b></span>
          <span><b>*</b>Italic<b>*</b></span>
          <span><b>[</b>Link<b>](</b>url<b>)</b></span>
          <span><b>></b> Blockquote</span>
        </div>
        <div class="cheatsheet-group">
          <span class="group-title">Lists & Tasks</span>
          <span><b>-</b> List Item</span>
          <span><b>- [ ]</b> Todo</span>
          <span><b>- [x]</b> Done</span>
        </div>
        <div class="cheatsheet-group">
          <span class="group-title">Advanced</span>
          <span><b>|</b> Table <b>|</b></span>
          <span><b>---</b> Separator</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.editor-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  width: 100%;
  height: 100%;
}

.editor-header {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  height: 64px;
}

.edit-buttons-group {
  display: flex;
  align-items: stretch;
  gap: 0.5rem;
  padding-top:.2rem;
  padding-bottom:.2rem;
  height: 100%;
}

.edit-buttons {
  height: 100%;
  width:56px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.editor-container {
  flex: 1;
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
}

.editor-cheatsheet {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  background-color: var(--bg-01);
  padding: 0.6rem 1rem;
  border-radius: 6px;
  border: var(--border-width) solid var(--bg-02);
  font-size: 0.8rem;
  color: var(--text-01);
}

.cheatsheet-group {
  display: flex;
  width: max-content;
  flex-direction: column;
  gap: 0.6rem;
  margin-bottom: 1rem;
  padding-right: .5rem;
}

.group-title {
  color: var(--text-02);
  font-weight: 600;
  text-transform: uppercase;
  font-size: 0.75rem;
  margin-right: 0.2rem;
}

.editor-cheatsheet b {
  color: var(--color-blue);
  font-family: 'Courier New', Courier, monospace;
}

.editor-textarea {
  width: 100%;
  height: 100%;
  font-family: 'Courier New', Courier, monospace;
  font-size: 1rem;
  line-height: 1.5;
  padding: 1rem;
  resize: none;
  background-color: var(--bg-00);
  color: var(--text-00);
  border: var(--border-width) solid var(--bg-02);
  border-radius: 6px;
}
</style>