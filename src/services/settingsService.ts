import { invoke } from "@tauri-apps/api/core";

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
    ollama: OllamaSettings;
    markdown: MarkdownSettings;
}

export const settingsService = {
  // Load global configuration from disk
  async load(): Promise<AppSettings> {
    return await invoke<AppSettings>("load_app_settings");
  },

  // Save global configuration to disk
  async save(settings: AppSettings): Promise<void> {
    await invoke("save_app_settings", { settings });
  }
};