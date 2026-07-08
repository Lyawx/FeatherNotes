<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "save"): void;
  (e: "cancel"): void;
}>();

// Liaison bidirectionnelle propre pour le v-model
const textContent = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const defaultPlaceholder = `# Header 1\n\nWrite your markdown here...\n\n- [ ] Todo item\n- [x] Done item`;
</script>

<template>
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
</template>

<style scoped>

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
  min-height: 420px;
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