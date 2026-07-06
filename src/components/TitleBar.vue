<script setup lang="ts">
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

// Récupère l'instance globale de la vue/fenêtre active
const appWindow = getCurrentWebviewWindow();

const minimizeWindow = async () => {
  await appWindow.minimize();
};

const maximizeWindow = async () => {
  await appWindow.toggleMaximize();
};

const closeWindow = async () => {
  await appWindow.close();
};
</script>

<template>
  <div data-tauri-drag-region class="custom-titlebar">
    <div class="titlebar-logo">
      <span>FeatherNotes</span>
    </div>
    
    <div class="titlebar-actions">
      <button class="titlebar-btn" @click="minimizeWindow">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect fill="currentColor" x="1" y="5" width="10" height="1.5"/></svg>
      </button>
      <button class="titlebar-btn" @click="maximizeWindow">
        <svg width="12" height="12" viewBox="0 0 12 12"><path fill="none" stroke="currentColor" stroke-width="1.2" d="M2,2 H10 V10 H2 Z"/></svg>
      </button>
      <button class="titlebar-btn close-btn" @click="closeWindow">
        <svg width="12" height="12" viewBox="0 0 12 12"><path fill="none" stroke="currentColor" stroke-width="1.2" d="M2,2 L10,10 M10,2 L2,10"/></svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.custom-titlebar {
  height: 32px;
  background: var(--bg-01);
  display: flex;
  justify-content: space-between;
  align-items: center;
  user-select: none;
  border-bottom: var(--border-width) solid var(--bg-02);
}

.titlebar-logo {
  padding-left: 12px;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-01);
  pointer-events: none; /* Évite de bloquer le drag sur le texte */
}

.titlebar-actions {
  display: flex;
  height: 100%;
}

.titlebar-btn {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 44px;
  height: 100%;
  background: transparent;
  border: none;
  color: var(--text-01);
  cursor: pointer;
  transition: background 0.1s ease;
}

.titlebar-btn:hover {
  background: var(--bg-active);
}

.close-btn:hover {
  background: var(--color-red) !important;
  color: #ffffff !important;
}
</style>