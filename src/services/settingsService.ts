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
  theme: string;
  ollama: OllamaSettings;
  markdown: MarkdownSettings;
  urgent_threshold_days: number;
  warning_threshold_days: number;
}

export const settingsService = {
  async loadSettings(): Promise<AppSettings> {
    return await invoke<AppSettings>('load_app_settings');
  },

  async saveSettings(settings: AppSettings): Promise<void> {
    await invoke('save_app_settings', { settings });
  }
};