import { invoke } from "@tauri-apps/api/core";
import { settingsService } from "./settingsService";

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: FileNode[];
}

export const logbookService = {
  async getLogbookFiles(): Promise<FileNode[]> {
    const settings = await settingsService.loadSettings();
    const vaultPath = settings.markdown.markdown_use_custom_path && settings.markdown.markdown_custom_path
      ? settings.markdown.markdown_custom_path
      : settings.markdown.markdown_default_path;

    if (!vaultPath) {
      throw new Error("Aucun chemin de Vault configuré.");
    }

    try {
      const rootTree = await invoke<FileNode>("get_vault_tree", { vaultPath });
      const logbookNode = rootTree.children.find(
        (child) => child.is_dir && child.name.toLowerCase() === "logbook"
      );

      if (logbookNode) {
        return logbookNode.children.filter((child) => !child.is_dir);
      }
    } catch (err) {
      console.warn("Le Vault n'existe probablement pas encore, création en cours...", err);
    }

    const separator = vaultPath.includes("/") ? "/" : "\\";
    const logbookPath = `${vaultPath}${separator}Logbook`;
    
    await invoke("create_vault_directory", { dirPath: logbookPath });

    try {
      const freshTree = await invoke<FileNode>("get_vault_tree", { vaultPath });
      const newLogbook = freshTree.children.find(
        (child) => child.is_dir && child.name.toLowerCase() === "logbook"
      );
      return newLogbook ? newLogbook.children.filter((child) => !child.is_dir) : [];
    } catch {
      return [];
    }
  },

  async getFileHtml(filePath: string): Promise<string> {
    return await invoke<string>("convert_md_to_html", { filePath });
  },

  async getFileRawText(filePath: string): Promise<string> {
    return await invoke<string>("read_raw_markdown", { filePath });
  },

  async saveFileRawText(filePath: string, content: string): Promise<void> {
    await invoke("save_markdown_file", { filePath, content });
  }

};