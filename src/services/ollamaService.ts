import { invoke } from "@tauri-apps/api/core";

export type OllamaStatus = 'connected' | 'disconnected' | 'connecting';

export const ollamaService = {
  // Check if Ollama engine is alive
  async checkStatus(): Promise<boolean> {
    return await invoke<boolean>("check_ollama_status");
  },

  // Boot up Ollama detached backend process
  async startProcess(): Promise<void> {
    await invoke("start_ollama_process");
  },

  // Kill stray Ollama background instances
  async stopProcess(): Promise<void> {
    await invoke("stop_ollama_process");
  },

  // Fetch local models installed on the machine
  async fetchModels(): Promise<string[]> {
    return await invoke<string[]>("fetch_models");
  },

  // Send the raw data dump to the processing pipeline
  async submitBrainDump(model: string, dump: string): Promise<string> {
    return await invoke<string>("process_brain_dump", {
      selectedModel: model,
      rawDump: dump
    });
  }
};