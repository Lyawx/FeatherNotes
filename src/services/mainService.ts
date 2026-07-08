import { settingsService } from "./settingsService";
import { ollamaService } from "./ollamaService";
import { logbookService } from "./logbookService";  
import { themeService } from "./themeService.ts"
import { itemService } from "./itemService.ts";
import { moveWindow, Position } from '@tauri-apps/plugin-positioner';
import { taskService } from "./taskService.ts";
export type { AppSettings } from "./settingsService";
export type { OllamaStatus } from "./ollamaService";
export type { FileNode } from "../services/logbookService";
export type { TaskProject, Milestone } from '../services/taskService';

export const mainService = {
  settings: settingsService,
  ollama: ollamaService,
  logbook: logbookService,
  themes: themeService,
  tasks: taskService,
  item: itemService,

  async centerWindow(): Promise<void> {
    try {
      await moveWindow(Position.Center);
    } catch (err) {
      console.error('Failed to center application window via plugin:', err);
    }
  }
};