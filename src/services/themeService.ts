import { invoke } from '@tauri-apps/api/core';
import { settingsService } from './settingsService';

export const themeService = {
  async getAvailableThemes(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_available_themes');
    } catch (error) {
      console.error('Impossible de récupérer la liste des thèmes:', error);
      return ['default'];
    }
  },
  async applyTheme(themeName: string): Promise<void> {
    let styleElement = document.getElementById('feather-runtime-theme') as HTMLStyleElement;

    // Si on repasse sur le thème par défaut, on supprime le CSS dynamique
    if (themeName === 'default') {
      if (styleElement) {
        styleElement.remove();
      }
      return;
    }

    // Sinon, on crée la balise si elle n'existe pas encore
    if (!styleElement) {
      styleElement = document.createElement('style');
      styleElement.id = 'feather-runtime-theme';
      document.head.appendChild(styleElement);
    }

    try {
      // On demande au backend le contenu brut du fichier CSS demandé
      const cssContent = await invoke<string>('load_theme_raw', { themeName });
      styleElement.textContent = cssContent;
    } catch (error) {
      console.error(`Impossible de charger le thème distant "${themeName}":`, error);
    }
  },

  async init(): Promise<void> {
    try {
      const settings = await settingsService.loadSettings();
      if (settings && settings.theme) {
        await this.applyTheme(settings.theme);
      }
    } catch (error) {
      console.error('Échec de l\'initialisation du thème au démarrage:', error);
    }
  }
};