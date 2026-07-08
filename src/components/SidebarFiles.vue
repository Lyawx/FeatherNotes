<script setup lang="ts" generic="T extends { path: string, name: string, createdAt?: string | Date | number, updatedAt?: string | Date | number }">
import { ref, nextTick, computed, watch } from 'vue';

type SortKey = 'alpha' | 'created' | 'updated' | string;

interface CustomSort {
  label: string;
  compareFn: (a: T, b: T) => number;
}

const props = defineProps<{
  title: string;
  items: T[];
  isLoading?: boolean;
  emptyMessage?: string;
  emptyHint?: string;
  customSorts?: Record<string, CustomSort>;
}>();

const emit = defineEmits<{
  (e: 'create-click'): void;
  (e: 'rename', payload: { item: T, newName: string }): void;
  (e: 'update:sorted', payload: any[]): void;
}>();

const editingId = ref<string | null>(null);
const tempName = ref("");
const inputRef = ref<HTMLInputElement | null>(null);
const currentSort = ref<SortKey>('alpha');
const isDirectionNormal = ref<boolean>(true);

const sortedItems = computed(() => {
  const list = [...props.items];
  
  if (props.customSorts && props.customSorts[currentSort.value]) {
    list.sort(props.customSorts[currentSort.value].compareFn);
  } else {
    switch (currentSort.value) {
      case 'alpha':
        list.sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' }));
        break;
      
      case 'created':
        list.sort((a, b) => {
          const dateA = new Date(a.createdAt || (a as any).birthtime || (a as any).created_at || (a as any).mtimeMs || 0).getTime();
          const dateB = new Date(b.createdAt || (b as any).birthtime || (b as any).created_at || (b as any).mtimeMs || 0).getTime();
          return dateB - dateA;
        });
        break;

      case 'updated':
        list.sort((a, b) => {
          const dateA = new Date(a.updatedAt || (a as any).mtime || (a as any).updated_at || (a as any).mtimeMs || 0).getTime();
          const dateB = new Date(b.updatedAt || (b as any).mtime || (b as any).updated_at || (b as any).mtimeMs || 0).getTime();
          return dateB - dateA;
        });
        break;
    }
  }

  if (!isDirectionNormal.value) {
    list.reverse();
  }

  return list;
});

const startCreate = () => emit('create-click');

const toggleDirection = () => {
  isDirectionNormal.value = !isDirectionNormal.value;
};

const startEdit = async (item: T) => {
  editingId.value = item.path;
  tempName.value = item.name.replace('.md', '');
  await nextTick();
  inputRef.value?.focus();
  inputRef.value?.select();
};

const confirm = (item: T) => {
  if (tempName.value.trim()) {
    emit('rename', { item, newName: tempName.value });
  }
  editingId.value = null;
};

const cancel = () => {
  editingId.value = null;
};

watch(sortedItems, (newVal) => {
  emit('update:sorted', newVal);
}, { immediate: true, deep: true });
</script>

<template>
  <aside class="sidebar-files">
    <div class="header">
      <h3>{{ title }}</h3>
      <button @click.stop="startCreate" class="icon-btn">
        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M12.75 9C12.75 8.58579 12.4142 8.25 12 8.25C11.5858 8.25 11.25 8.58579 11.25 9L11.25 11.25H9C8.58579 11.25 8.25 11.5858 8.25 12C8.25 12.4142 8.58579 12.75 9 12.75H11.25V15C11.25 15.4142 11.5858 15.75 12 15.75C12.4142 15.75 12.75 15.4142 12.75 15L12.75 12.75H15C15.4142 12.75 15.75 12.4142 15.75 12C15.75 11.5858 15.4142 11.25 15 11.25H12.75V9Z" fill="currentColor"></path>
          <path d="M12 1.25C6.06294 1.25 1.25 6.06294 1.25 12C1.25 17.9371 6.06294 22.75 12 22.75C17.9371 22.75 22.75 17.9371 22.75 12C22.75 6.06294 17.9371 1.25 12 1.25ZM2.75 12C2.75 6.89137 6.89137 2.75 12 2.75C17.1086 2.75 21.25 6.89137 21.25 12C21.25 17.1086 17.1086 21.25 12 21.25C6.89137 21.25 2.75 17.1086 2.75 12Z" fill="currentColor"></path>
        </svg>
      </button>
    </div>

    <div class="sort-wrapper-row">
      <div class="select-container">
        <select v-model="currentSort" class="dropdown">
          <option value="alpha">A-Z</option>
          <option value="updated">Last Updated</option>
          <option value="created">Last Created</option>
          <template v-if="customSorts">
            <option v-for="(sort, key) in customSorts" :key="key" :value="key">
              {{ sort.label }}
            </option>
          </template>
        </select>
      </div>
      <button @click="toggleDirection" class="direction-btn" :class="{ 'is-reversed': !isDirectionNormal }">
        <svg xmlns="http://www.w3.org/2000/svg" width="1.2em" height="1.2em" viewBox="0 0 24 24">
          <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19V5m4 10l-4 4l-4-4" />
        </svg>
      </button>
    </div>

    <hr class="ui-divider" />

    <div v-if="isLoading" class="ui-spinner-container">...</div>
    <div v-else-if="items.length === 0" class="empty-state">...</div>

    <div v-else class="file-list">
      <div v-for="item in sortedItems" :key="item.path" class="item-row">
        <div class="name-container">
          <input 
            v-if="editingId === item.path"
            ref="inputRef"
            v-model="tempName" 
            @keyup.enter="confirm(item)"
            @keyup.esc="cancel"
            @blur="confirm(item)"
            class="seamless-input"
          />
          <div v-else class="slot-wrapper">
            <slot name="item" :item="item"></slot>
          </div>
        </div>

        <button 
          @click="editingId === item.path ? confirm(item) : startEdit(item)" 
          class="action-btn"
          :class="{ 'is-confirm': editingId === item.path }"
        >
          <svg v-if="editingId === item.path" xmlns="http://www.w3.org/2000/svg" width="1.1em" height="1.1em" viewBox="0 0 24 24">
            <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 6L9 17l-5-5"/>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
            <path fill="currentColor" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
          </svg>
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar-files {
  width: 260px;
  min-width: 260px;
  max-width: 260px;
  height: 100%;
  box-sizing: border-box;
  flex-shrink: 0;
  background-color: var(--bg-01);
  border-right: var(--border-width) solid var(--bg-02);
  padding: 1.5rem 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header h3 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-00);
}

.sort-wrapper-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.select-container {
  flex: 1;
  position: relative;
}

.select-container::after {
  content: "";
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 0.8rem;
  height: 0.8rem;
  background-color: var(--text-02);
  clip-path: polygon(15% 30%, 85% 30%, 50% 75%);
  pointer-events: none;
  transition: background-color 0.15s ease, transform 0.2s ease;
}

.select-container:hover::after {
  background-color: var(--text-00);
}

.direction-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.45rem;
  border-radius: 6px;
  border: var(--border-width) solid var(--bg-02);
  background-color: var(--bg-00);
  color: var(--text-01);
  cursor: pointer;
  transition: all 0.15s ease;
}

.direction-btn:hover {
  background-color: var(--bg-02);
  color: var(--text-00);
}

.direction-btn svg {
  transition: transform 0.2s ease;
}

.direction-btn.is-reversed svg {
  transform: rotate(180deg);
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: none;
  cursor: pointer;
}

.icon-btn svg {
  width: 26px;
  height: 26px;
  color: var(--text-00);
}

.icon-btn:hover svg {
  color: var(--color-blue);
}

.item-row {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  box-sizing: border-box;
}

.name-container {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
}

.slot-wrapper {
  width: 100%;
  overflow: hidden;
}

.seamless-input {
  width: 100%;
  border: none;
  background: transparent;
  padding: 0;
  margin: 0;
  font-family: inherit;
  font-size: inherit;
  font-weight: inherit;
  color: inherit;
  outline: none;
  box-sizing: border-box;
}

.action-btn {
  cursor: pointer;
  background: none;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-01);
  transition: color 100ms ease-in-out;
  padding: 4px;
}

.action-btn:hover {
  color: var(--color-blue);
}

.action-btn.is-confirm {
  color: var(--color-green, #28a745);
}

.ui-divider {
  border: none;
  border-top: var(--border-width) solid var(--bg-02);
  margin: 0;
  width: 100%;
}
</style>