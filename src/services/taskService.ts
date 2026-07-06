import { invoke } from '@tauri-apps/api/core';

export interface TaskItem {
  id: number;
  text: string;
  completed: boolean;
  due_date: string | null; // <-- Ajouté
}

export interface Milestone {
  name: string;
  path: string;
  date: string | null;     // <-- Ajouté
  tasks: TaskItem[];
}

export interface TaskProject {
  name: string;
  path: string;
  milestones: Milestone[];
}

export const taskService = {
  async getTasksStructure(): Promise<TaskProject[]> {
    return await invoke<TaskProject[]>('get_tasks_structure');
  },

  async toggleTask(filePath: string, taskIndex: number, completed: boolean): Promise<void> {
    await invoke('toggle_task_in_file', { filePath, taskIndex, completed });
  }
};