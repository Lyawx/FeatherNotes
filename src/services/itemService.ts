import { invoke } from "@tauri-apps/api/core";
import { settingsService } from "./settingsService";

export interface SidebarItem {
  path: string;
  name: string;
  createdAt: number;
  updatedAt: number;
}

export const itemService = {
  async renameItem(oldPath: string, newName: string): Promise<void> {
    await invoke("rename_item", { oldPath, newName });
  },

  async getFileHtml(filePath: string): Promise<string> {
    return await invoke<string>("convert_md_to_html", { filePath });
  },

  async getFileRawText(filePath: string): Promise<string> {
    return await invoke<string>("read_raw_markdown", { filePath });
  },

  async saveFileRawText(filePath: string, content: string): Promise<void> {
    await invoke("save_markdown_file", { filePath, content });
  },

  async getDirectoryPath(dirName: string): Promise<string> {
      const settings = await settingsService.loadSettings();
      const vaultPath = settings.markdown.markdown_use_custom_path && settings.markdown.markdown_custom_path
        ? settings.markdown.markdown_custom_path
        : settings.markdown.markdown_default_path;
  
      if (!vaultPath) {
        throw new Error("Aucun chemin de Vault configuré.");
      }
  
      const separator = vaultPath.includes("/") ? "/" : "\\";
      return `${vaultPath}${separator}` + dirName;
  },

  async createDir(dirPath: string): Promise<void> {
    await invoke("create_dir", { dirPath });
  },
};
