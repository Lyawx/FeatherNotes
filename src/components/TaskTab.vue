<script setup lang="ts">
import { ref, onMounted, computed, nextTick } from 'vue';
import { mainService } from '../services/mainService';
import { settingsService } from '../services/settingsService';
import type { TaskProject, Milestone } from '../services/mainService';
import SidebarFiles from './SidebarFiles.vue';

const projects = ref<TaskProject[]>([]);
const sortedProjectsList = ref<TaskProject[]>([]);
const expandedProjects = ref<Record<string, boolean>>({});
const expandedMilestones = ref<Record<string, boolean>>({});
const selectedProjects = ref<Record<string, boolean>>({});
const urgentThresholdDays = ref(1);
const warningThresholdDays = ref(3);

const editingProjectPath = ref<string | null>(null);
const tempProjectName = ref("");
const renameInputRef = ref<HTMLInputElement | null>(null);

const updateSortedList = (newList: TaskProject[]) => {
  sortedProjectsList.value = newList;
};

const loadTasks = async () => {
  try {
    const data = await mainService.tasks.getTasksStructure();
    projects.value = data;

    data.forEach(project => {
      if (selectedProjects.value[project.path] === undefined) {
        selectedProjects.value[project.path] = true;
      }
    });
  } catch (err) {
    console.error('Error loading tasks:', err);
  }
};

const filteredProjects = computed(() => {
  const baseList = sortedProjectsList.value.length > 0 ? sortedProjectsList.value : projects.value;
  return baseList.filter((project: TaskProject) => selectedProjects.value[project.path]);
});

const loadSettings = async () => {
  try {
    const settings = await settingsService.loadSettings();
    urgentThresholdDays.value = settings.urgent_threshold_days;
    warningThresholdDays.value = settings.warning_threshold_days;
  } catch (err) {
    console.error('Error loading priority settings:', err);
  }
};

onMounted(async () => {
  await loadSettings();
  await loadTasks();
});

const toggleProject = (projectPath: string) => {
  if (editingProjectPath.value === projectPath) return;
  expandedProjects.value[projectPath] = !expandedProjects.value[projectPath];
};

const toggleMilestone = (milestonePath: string) => {
  expandedMilestones.value[milestonePath] = !expandedMilestones.value[milestonePath];
};

const handleTaskToggle = async (filePath: string, taskId: number, completed: boolean) => {
  projects.value = projects.value.map(project => {
    return {
      ...project,
      milestones: project.milestones.map(milestone => {
        if (milestone.path !== filePath) return milestone;
        return {
          ...milestone,
          tasks: milestone.tasks.map(task => {
            if (task.id !== taskId) return task;
            return { ...task, completed };
          })
        };
      })
    };
  });

  try {
    await mainService.tasks.toggleTask(filePath, taskId, completed);
  } catch (err) {
    console.error('Error toggling task:', err);
    await loadTasks();
  }
};

const calculatePriority = (dateStr: string | null): 'overdue' | 'urgent' | 'warning' | 'normal' => {
  if (!dateStr) return 'normal';

  const targetDate = new Date(dateStr);
  const today = new Date();

  today.setHours(0, 0, 0, 0);
  targetDate.setHours(0, 0, 0, 0);

  const diffTime = targetDate.getTime() - today.getTime();
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

  if (diffDays < 0) return 'overdue';
  if (diffDays <= urgentThresholdDays.value) return 'urgent';
  if (diffDays <= warningThresholdDays.value) return 'warning';
  return 'normal';
};

const isMilestoneCompleted = (milestone: Milestone): boolean => {
  if (milestone.tasks.length === 0) return false;
  return milestone.tasks.every(t => t.completed);
};

const getMilestoneProgress = (milestone: Milestone): number => {
  if (!milestone.tasks || milestone.tasks.length === 0) return 0;
  const completed = milestone.tasks.filter(t => t.completed).length;
  return Math.round((completed / milestone.tasks.length) * 100);
};

const getProjectProgress = (project: TaskProject): number => {
  let totalTasks = 0;
  let completedTasks = 0;

  project.milestones.forEach(m => {
    if (m.tasks) {
      totalTasks += m.tasks.length;
      completedTasks += m.tasks.filter(t => t.completed).length;
    }
  });

  if (totalTasks === 0) return 0;
  return Math.round((completedTasks / totalTasks) * 100);
};

const startRename = async (project: TaskProject) => {
  editingProjectPath.value = project.path;
  tempProjectName.value = project.name;
  await nextTick();
  renameInputRef.value?.focus();
  renameInputRef.value?.select();
};

const confirmRename = async (project: TaskProject) => {
  const newName = tempProjectName.value.trim();
  if (!newName || newName === project.name) {
    editingProjectPath.value = null;
    return;
  }

  try {
    const wasSelected = selectedProjects.value[project.path];
    const isExpanded = expandedProjects.value[project.path];
    
    await mainService.item.renameItem(project.path, newName);
    await loadTasks();
    
    const updatedProject = projects.value.find(p => p.name === newName || p.path.endsWith(newName));
    if (updatedProject) {
      if (wasSelected !== undefined) {
        selectedProjects.value[updatedProject.path] = wasSelected;
        delete selectedProjects.value[project.path];
      }
      if (isExpanded !== undefined) {
        expandedProjects.value[updatedProject.path] = isExpanded;
        delete expandedProjects.value[project.path];
      }
    }
  } catch (error) {
    console.error("Error renaming project:", error);
  } finally {
    editingProjectPath.value = null;
  }
};

const cancelRename = () => {
  editingProjectPath.value = null;
};

const PRIORITY_WEIGHTS = {
  overdue: 4,
  urgent: 3,
  warning: 2,
  normal: 1
};

const getProjectPriorityScore = (project: TaskProject): number => {
  const activeMilestones = project.milestones.filter(m => m.date && !isMilestoneCompleted(m));

  return activeMilestones.reduce((sum, milestone) => {
    const priority = calculatePriority(milestone.date);
    return sum + PRIORITY_WEIGHTS[priority];
  }, 0);
};

const taskCustomSorts = {
  priority: {
    label: 'Priority score',
    compareFn: (a: TaskProject, b: TaskProject) => {
      const scoreA = getProjectPriorityScore(a);
      const scoreB = getProjectPriorityScore(b);

      if (scoreB !== scoreA) return scoreB - scoreA;

      return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
    }
  }
};

const createNewProject = async () => {
  const fileName = prompt("File name :");
  if (!fileName) return;
  try {
    const dirPath = await mainService.item.getDirectoryPath('Tasks');
    const separator = dirPath.includes("/") ? "/" : "\\";
    let fullPath = `${dirPath}${separator}${fileName}`;
    await mainService.item.createDir(fullPath);
    await loadTasks();
  } catch (err) {
    console.error("Erreur :", err);
  }
};
</script>

<template>
  <div id="tasks-wrapper">
    <SidebarFiles 
      @create-click="createNewProject" 
      title="Displayed Projects"
      :items="projects"
      :customSorts="taskCustomSorts"
      emptyMessage="No project files found in the Tasks folder."
      emptyHint="Add a new project to see it displayed here."
      @update:sorted="updateSortedList"
    >
      <template #item="{ item: project }">
        <div class="project-filter-item">
          <label class="filter-label" @click.stop>
            <input type="checkbox" v-model="selectedProjects[project.path]" class="eye-checkbox" />
            <span class="eye-icon-wrapper">
              <svg class="eye-open" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
              <svg class="eye-closed" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
                <line x1="1" y1="1" x2="23" y2="23" />
              </svg>
            </span>
            <span class="filter-text" :class="{ 'text-muted': !selectedProjects[project.path] }">
              {{ project.name }}
            </span>
          </label>
        </div>
      </template>
    </SidebarFiles>

    <div id="tasks-panel" class="tab-panel">
      <h1>Project Tasks</h1>
      <p class="tab-desc">Manage your milestones and checkboxes directly synced with your markdown vault.</p>

      <div v-if="filteredProjects.length === 0" class="empty-state">
        No project selected or found.
      </div>

      <div v-else class="projects-list">
        <div v-for="project in filteredProjects" :key="project.path" class="project-container">

          <div class="project-header" @click="toggleProject(project.path)">
            <span class="chevron" :class="{ 'rotated': expandedProjects[project.path] }">▶</span>
            
            <div v-if="editingProjectPath === project.path" class="project-rename-container" @click.stop>
              <input 
                ref="renameInputRef"
                v-model="tempProjectName"
                class="project-rename-input"
                @keyup.enter="confirmRename(project)"
                @keyup.esc="cancelRename"
              />
              <button class="project-rename-cancel-btn" @click="cancelRename">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
            <h2 v-else class="project-title">{{ project.name }}</h2>

            <button 
              v-if="editingProjectPath !== project.path"
              class="project-rename-btn" 
              @click.stop="startRename(project)"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                <path fill="currentColor" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
              </svg>
            </button>

            <div class="project-header-right">
              <span class="badge-project-score" v-if="getProjectPriorityScore(project) > 0">
                Priority score : {{ getProjectPriorityScore(project) }}
              </span>

              <div class="project-progress-layout">
                <span class="progress-text-large">{{ getProjectProgress(project) }}%</span>
                <div class="progress-bar-container-large">
                  <div class="progress-bar-fill" :style="{ width: getProjectProgress(project) + '%' }"></div>
                </div>
              </div>
            </div>
          </div>

          <div v-show="expandedProjects[project.path]" class="milestones-list">
            <div v-for="milestone in project.milestones" :key="milestone.path" class="milestone-container">
              <div class="milestone-header" @click="toggleMilestone(milestone.path)">
                <span class="chevron" :class="{ 'rotated': expandedMilestones[milestone.path] }">▶</span>

                <div class="milestone-progress-circle-wrapper">
                  <svg class="progress-circle" viewBox="0 0 36 36">
                    <path class="circle-bg" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" />
                    <path class="circle-fill" :stroke-dasharray="getMilestoneProgress(milestone) + ', 100'" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" />
                  </svg>
                  <span class="circle-text">{{ getMilestoneProgress(milestone) }}%</span>
                </div>

                <span v-if="milestone.date && !isMilestoneCompleted(milestone)" class="badge-priority" :class="calculatePriority(milestone.date)">
                  {{ calculatePriority(milestone.date) }}
                </span>

                <h3 class="milestone-title">{{ milestone.name }}</h3>
                <span v-if="milestone.date" class="badge-title-date">{{ milestone.date }}</span>
              </div>

              <div v-show="expandedMilestones[milestone.path]" class="tasks-list">
                <div v-for="task in milestone.tasks" :key="task.id" class="task-row">
                  <label class="task-checkbox-wrapper">
                    <input type="checkbox" :checked="task.completed" @change="handleTaskToggle(milestone.path, task.id, !task.completed)" />
                    <span v-if="task.due_date && !task.completed" class="badge-priority" :class="calculatePriority(task.due_date)">
                      {{ calculatePriority(task.due_date) }}
                    </span>
                    <span class="task-text" :class="{ 'task-done': task.completed }">
                      {{ task.text }}
                    </span>
                    <span v-if="task.due_date" class="badge-date date" :class="{ 'date-done': task.completed }">
                      {{ task.due_date }}
                    </span>
                  </label>
                </div>
                <div v-if="milestone.tasks.length === 0" class="empty-tasks">
                  No checkboxes found in this file.
                </div>
              </div>
            </div>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
#tasks-wrapper {
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
}

#tasks-panel {
  width: 100%;
}

.project-container {
  background: var(--bg-01);
  border: var(--border-width) solid var(--bg-02);
  border-radius: 6px;
  margin-bottom: 1rem;
  overflow: hidden;
}

.project-header {
  display: flex;
  align-items: center;
  padding: 0.85rem 1.25rem;
  cursor: pointer;
  background: var(--bg-02);
  user-select: none;
  gap: 0.5rem;
}

.project-header-right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 1.5rem;
  flex-shrink: 0;
}

.project-title {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 600;
  color: var(--text-00);
}

.project-rename-btn {
  cursor: pointer;
  background: none;
  border: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-02);
  opacity: 0;
  transition: opacity 0.15s ease, color 0.1s ease;
  padding: 6px;
  border-radius: 4px;
}

.project-header:hover .project-rename-btn {
  opacity: 1;
}

.project-rename-btn:hover {
  color: var(--color-blue);
  background-color: var(--bg-01);
}

.project-rename-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-grow: 1;
  max-width: 450px;
}

.project-rename-input {
  font-size: 1.15rem;
  font-weight: 600;
  font-family: inherit;
  color: var(--text-00);
  background: var(--bg-00);
  border: 1px solid var(--color-blue);
  border-radius: 4px;
  padding: 2px 8px;
  outline: none;
  flex-grow: 1;
  width: 100%;
}

.project-rename-cancel-btn {
  background: none;
  border: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-02);
  padding: 6px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.15s, color 0.15s;
}

.project-rename-cancel-btn:hover {
  color: var(--color-red, #ef4444);
  background-color: var(--bg-01);
}

.badge-project-score {
  font-size: 1.15rem;
  font-weight: 600;
  font-family: inherit;
  color: var(--color-blue);
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
}

.chevron {
  font-size: 0.75rem;
  margin-right: 0.25rem;
  transition: transform 0.2s ease;
  color: var(--text-02);
  flex-shrink: 0;
}

.chevron.rotated {
  transform: rotate(90deg);
}

.milestones-list {
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.milestone-container {
  border-left: 2px solid var(--bg-02);
  padding-left: 0.5rem;
}

.milestone-header {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 0.75rem;
  padding: 0.4rem 0.5rem;
  cursor: pointer;
  user-select: none;
}

.milestone-title {
  margin: 0;
  font-size: 0.95rem;
  color: var(--text-01);
  white-space: nowrap;
}

.tasks-list {
  padding: 0.5rem 0 0.5rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.task-row {
  display: flex;
  align-items: center;
}

.task-checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
  width: 100%;
}

.task-text {
  font-size: 0.9rem;
  color: var(--text-00);
}

.task-done {
  text-decoration: line-through;
  color: var(--text-02);
}

.badge-title-date {
  font-size: 0.9rem;
  padding: 0.15rem 0.4rem;
  text-align: right;
  border-radius: 4px;
  white-space: nowrap;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 600;
  color: var(--text-02);
  margin-left: auto;
}

.badge-date {
  font-size: 0.75rem;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  font-weight: 600;
  white-space: nowrap;
  text-align: right;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-02);
  margin-left: auto;
}

.date-done {
  text-decoration: line-through;
}

.project-filter-item {
  width: 100%;
  display: flex;
  align-items: center;
}

.filter-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  cursor: pointer;
  padding: 0.2rem 0;
}

.filter-label * {
  transition: all 0.2s ease;
}

.filter-text {
  font-size: 0.9rem;
  color: var(--text-01);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.2; 
  transition: color 0.2s ease;
}

.project-filter-item:hover {
  background-color: var(--bg-02);
}

.filter-text.text-muted {
  color: var(--text-02);
  opacity: 0.6;
}

.eye-checkbox {
  position: absolute;
  opacity: 0;
  cursor: pointer;
  height: 0;
  width: 0;
}

.eye-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  color: var(--text-02);
  cursor: pointer;
}

.eye-icon-wrapper svg {
  width: 100%;
  height: 100%;
  display: block;
}

.eye-checkbox ~ .eye-icon-wrapper .eye-closed {
  display: block;
}
.eye-checkbox ~ .eye-icon-wrapper .eye-open {
  display: none;
}

.eye-checkbox:checked ~ .eye-icon-wrapper .eye-closed {
  display: none;
}
.eye-checkbox:checked ~ .eye-icon-wrapper .eye-open {
  display: block;
  color: var(--text-00);
}

.filter-label:hover * {
  color: var(--color-blue);
}

.project-progress-layout {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.progress-bar-container-large {
  width: 120px;
  height: 8px;
  background-color: var(--bg-00);
  border-radius: 4px;
  overflow: hidden;
  border: var(--border-width) solid var(--bg-02);
}

.progress-bar-fill {
  height: 100%;
  background-color: var(--color-green, #10b981);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.progress-text-large {
  font-size: 0.85rem;
  font-family: monospace;
  font-weight: 600;
  color: var(--text-01);
  min-width: 38px;
  text-align: right;
}
</style>