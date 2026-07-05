<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { mainService } from "../services/mainService";
import type { OllamaStatus } from "../services/ollamaService";

const props = defineProps<{ ollamaStatus: OllamaStatus }>();
const emit = defineEmits<{ (e: 'triggerStart'): void }>();

const userInput = ref<string>('');
const aiResult = ref<string>('');
const isProcessing = ref<boolean>(false);
const installedModels = ref<string[]>([]);
const selectedModel = ref<string>('');

onMounted(() => { if (props.ollamaStatus === 'connected') fetchModels(); });
watch(() => props.ollamaStatus, (newStatus) => {
    if (newStatus === 'connected') fetchModels();
    else if (newStatus === 'disconnected') { installedModels.value = []; selectedModel.value = ''; }
});

const fetchModels = async () => {
    try {
        const models = await mainService.ollama.fetchModels();
        installedModels.value = models;
        if (models.length > 0 && !selectedModel.value) selectedModel.value = models[0];
    } catch (err) { console.error(err); }
};

const submitBrainDump = async () => {
    if (!userInput.value.trim() || !selectedModel.value) return;
    isProcessing.value = true;
    aiResult.value = "";
    try {
        aiResult.value = await mainService.ollama.submitBrainDump(selectedModel.value, userInput.value);
    } catch (error) { aiResult.value = "Error: " + error; }
    finally { isProcessing.value = false; }
};
</script>

<template>
    <div id="ai-assistant-panel" class="tab-panel">
        <div class="header-zone">
            <h1>AI Assistant</h1>
            <p>Dump your thoughts, raw ideas, or your entire day. The local AI will sort everything automatically.</p>
        </div>

        <div id="ai-workspace-container">
            <div class="container-light ai-column">
                <textarea v-model="userInput" placeholder="Type or paste your unstructured day logs..."
                    class="text-input-field" id="brain-textarea" :disabled="isProcessing"></textarea>

                <div class="action-bar">
                    <div :class="['engine-status-box', ollamaStatus]" @click="emit('triggerStart')">
                        <span v-if="ollamaStatus === 'connecting'" class="ui-spinner"></span>
                        <span v-else class="engine-status-dot"></span>
                        <span class="engine-status-txt">
                            Ollama: <span class="engine-state-highlight">{{ ollamaStatus }}</span>
                        </span>
                    </div>

                    <div class="controls-group">
                        <select v-model="selectedModel" class="model-select"
                            :disabled="ollamaStatus !== 'connected' || isProcessing">
                            <option value="" disabled>Select a model...</option>
                            <option v-for="model in installedModels" :key="model" :value="model">{{ model }}</option>
                        </select>

                        <button id="submit-dump-btn" class="base-btn" @click="submitBrainDump"
                            :disabled="ollamaStatus !== 'connected' || !userInput.trim() || isProcessing">
                            Process Brain Dump
                        </button>
                    </div>
                </div>
            </div>

            <div class="container-light ai-column">
                <h3>AI Classification Preview</h3>

                <div id="ai-result-box" class="container-dark">
                    <div v-if="isProcessing" class="ui-progress-container">
                        <div class="ui-progress-track">
                            <div class="ui-progress-bar"></div>
                        </div>
                        <span class="ui-progress-label">The model is processing your raw logs...</span>
                    </div>

                    <div v-else-if="aiResult" class="preview-content">
                        <pre style="white-space: pre-wrap; font-family: inherit; color: #fff;">{{ aiResult }}</pre>
                    </div>

                    <div v-else class="preview-placeholder">
                        Your processed tasks will appear here.
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
#ai-assistant-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
}

#ai-workspace-container {
    display: flex;
    flex-direction: row;
    gap: 1.5rem;
    margin-top: 1.5rem;
    height: 100%;
}

.ai-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
}

#brain-textarea {
    resize: none;
    height: 100%;
}

#ai-result-box {
    margin-bottom: 0;
    height: 100%;
}

.action-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-left: 0.5rem;
}

.controls-group {
    display: flex;
    gap: 0.75rem;
}

.model-select {
    background-color: rgba(0, 0, 0, 0.2);
    color: #fff;
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 0.5rem;
    border-radius: 6px;
    font-family: inherit;
}

#submit-dump-btn {
    background-color: #3b82f6;
    color: #fff;
}

#submit-dump-btn:hover:not(:disabled) {
    background-color: #2563eb;
    box-shadow: 0 0 12px rgba(59, 130, 246, 0.3);
}

.preview-placeholder {
    color: #4b5563;
    font-size: 0.9rem;
    font-style: italic;
}
</style>