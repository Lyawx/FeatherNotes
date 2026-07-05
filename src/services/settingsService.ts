import { invoke } from "@tauri-apps/api/core";

// Explicitly export the interface so other services/components can use it
export interface AppSettings {
  ollama: {
    ollama_custom_path: string;
    ollama_use_custom_path: boolean;
  };
  markdown: {
    markdown_custom_path: string;
    markdown_use_custom_path: boolean;
  };
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