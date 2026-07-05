import { invoke } from "@tauri-apps/api/core";
import { settingsService } from "./settingsService";

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: FileNode[];
}

export const logbookService = {
  // Récupère ou initialise le dossier Logbook et renvoie ses fichiers Markdown
  async getLogbookFiles(): Promise<FileNode[]> {
    // 1. On récupère le chemin racine du Vault depuis les settings
    const settings = await settingsService.load();
    const vaultPath = settings.markdown.markdown_use_custom_path && settings.markdown.markdown_custom_path
      ? settings.markdown.markdown_custom_path
      : settings.markdown.markdown_default_path;

    if (!vaultPath) {
      throw new Error("Aucun chemin de Vault configuré.");
    }

    // 2. On tente de lire l'arbre complet du Vault
    try {
      const rootTree = await invoke<FileNode>("get_vault_tree", { vaultPath });
      
      // On cherche si un dossier "Logbook" existe à la racine du Vault
      const logbookNode = rootTree.children.find(
        (child) => child.is_dir && child.name.toLowerCase() === "logbook"
      );

      if (logbookNode) {
        // Si trouvé, on extrait uniquement ses enfants qui se terminent par .md
        return logbookNode.children.filter((child) => !child.is_dir);
      }
    } catch (err) {
      console.warn("Le Vault n'existe probablement pas encore, création en cours...", err);
    }

    // 3. Failsafe : Si le dossier Logbook n'existe pas, on le crée nativement via Rust
    // On détermine le chemin absolu du dossier Logbook (détecte si Windows ou Unix pour le slash)
    const separator = vaultPath.includes("/") ? "/" : "\\";
    const logbookPath = `${vaultPath}${separator}Logbook`;
    
    await invoke("create_vault_directory", { dirPath: logbookPath });

    // On relit l'arbre tout neuf (qui sera vide au départ)
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

  // Charge et convertit un fichier Markdown précis en HTML
  async getFileHtml(filePath: string): Promise<string> {
    return await invoke<string>("convert_md_to_html", { filePath });
  }
};