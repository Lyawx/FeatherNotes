// Import sibling services from the same directory
import { settingsService } from "./settingsService";
import { ollamaService } from "./ollamaService";
import { logbookService } from "./logbookService";  
import { ThemeService } from "./themeService.ts"
// Import window positioning capabilities from the Tauri plugin
import { moveWindow, Position } from '@tauri-apps/plugin-positioner';

// Re-export types so components only need to import mainService
export type { AppSettings } from "./settingsService";
export type { OllamaStatus } from "./ollamaService";

export const MainService = {
  settings: settingsService,
  ollama: ollamaService,
  logbook: logbookService,
  themes: ThemeService,

  // --- WINDOW MANAGEMENT ---
  // Centers the primary application window viewport on the desktop screen
  async centerWindow(): Promise<void> {
    try {
      await moveWindow(Position.Center);
    } catch (err) {
      console.error('Failed to center application window via plugin:', err);
    }
  }
};