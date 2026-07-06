import { invoke } from '@tauri-apps/api/core';

export interface OllamaSettings {
  ollama_default_path: string;
  ollama_custom_path: string;
  ollama_use_custom_path: boolean;
}

export interface MarkdownSettings {
  markdown_default_path: string;
  markdown_custom_path: string;
  markdown_use_custom_path: boolean;
}

export interface AppSettings {
  theme: string; // <-- On ajoute le champ ici pour TypeScript
  ollama: OllamaSettings;
  markdown: MarkdownSettings;
}

export const settingsService = { // <-- Minuscule respectée
  async loadSettings(): Promise<AppSettings> {
    return await invoke<AppSettings>('load_app_settings');
  },

  async saveSettings(settings: AppSettings): Promise<void> {
    await invoke('save_app_settings', { settings });
  }
};