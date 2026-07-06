<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { mainService } from '../services/mainService';
import { settingsService } from '../services/settingsService';
import type { TaskProject, Milestone } from '../services/taskService';

const projects = ref<TaskProject[]>([]);
const expandedProjects = ref<Record<string, boolean>>({});
const expandedMilestones = ref<Record<string, boolean>>({});

const urgentThresholdDays = ref(1);
const warningThresholdDays = ref(3);

const loadTasks = async () => {
  try {
    projects.value = await mainService.tasks.getTasksStructure();
  } catch (err) {
    console.error('Erreur lors du chargement des tâches:', err);
  }
};

const loadSettings = async () => {
  try {
    const settings = await settingsService.loadSettings();
    // On utilise les clés en snake_case du Rust
    urgentThresholdDays.value = settings.urgent_threshold_days;
    warningThresholdDays.value = settings.warning_threshold_days;
  } catch (err) {
    console.error('Erreur lors du chargement des configurations de priorités:', err);
  }
};

onMounted(async () => {
  await loadSettings();
  await loadTasks();
});

const toggleProject = (projectPath: string) => {
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
    console.error('Erreur lors de la modification de la tâche:', err);
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
</script>

<template>
  <div id="tasks-panel" class="tab-panel">
    <h1>Project Tasks</h1>
    <p class="panel-subtitle">Manage your milestones and checkboxes directly synced with your markdown vault.</p>

    <div v-if="projects.length === 0" class="empty-state">
      No projects or task files found.
    </div>

    <div v-else class="projects-list">
      <div v-for="project in projects" :key="project.path" class="project-container">
        
        <div class="project-header" @click="toggleProject(project.path)">
          <span class="chevron" :class="{ 'rotated': expandedProjects[project.path] }">▶</span>
          <h2 class="project-title">{{ project.name }}</h2>
          
          <div class="progress-wrapper project-progress-layout">
            <div class="progress-bar-container-large">
              <div class="progress-bar-fill" :style="{ width: getProjectProgress(project) + '%' }"></div>
            </div>
            <span class="progress-text-large">{{ getProjectProgress(project) }}%</span>
          </div>
        </div>

        <div v-show="expandedProjects[project.path]" class="milestones-list">
          <div v-for="milestone in project.milestones" :key="milestone.path" class="milestone-container">
            
            <div class="milestone-header" @click="toggleMilestone(milestone.path)">
              <span class="chevron" :class="{ 'rotated': expandedMilestones[milestone.path] }">▶</span>
              
              <div class="milestone-progress-circle-wrapper">
                <svg class="progress-circle" viewBox="0 0 36 36">
                  <path class="circle-bg"
                    d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                  />
                  <path class="circle-fill"
                    :stroke-dasharray="getMilestoneProgress(milestone) + ', 100'"
                    d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                  />
                </svg>
                <span class="circle-text">{{ getMilestoneProgress(milestone) }}%</span>
              </div>

              <span v-if="milestone.date && !isMilestoneCompleted(milestone)" 
                    class="badge-priority" 
                    :class="calculatePriority(milestone.date)">
                {{ calculatePriority(milestone.date) }}
              </span>

              <h3 class="milestone-title">{{ milestone.name }}</h3>
                            
              <span v-if="milestone.date" class="badge-title-date">
                {{ milestone.date }}
              </span>
            </div>

            <div v-show="expandedMilestones[milestone.path]" class="tasks-list">
              <div v-for="task in milestone.tasks" :key="task.id" class="task-row">
                <label class="task-checkbox-wrapper">
                  <input 
                    type="checkbox" 
                    :checked="task.completed" 
                    @change="handleTaskToggle(milestone.path, task.id, !task.completed)"
                  />

                  <span v-if="task.due_date && !task.completed" 
                        class="badge-priority" 
                        :class="calculatePriority(task.due_date)">
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
</template>

<style scoped>
.panel-subtitle {
  color: var(--text-02);
  margin-bottom: 2rem;
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
}

.project-title {
  margin: 0;
  font-size: 1.15rem;
  color: var(--text-00);
}

.chevron {
  font-size: 0.75rem;
  margin-right: 0.75rem;
  transition: transform 0.2s ease;
  color: var(--text-02);
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

.badge-priority {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.1rem 0.35rem;
  border-radius: 3px;
  font-weight: 600;
  white-space: nowrap;
}

.badge-priority.overdue {
  background-color: var(--color-red-surface);
  color: var(--color-red);
  border: 1px solid var(--color-red);
}

.badge-priority.urgent {
  background-color: #3a2222;
  color: #ff8585;
  border: 1px solid #b94a4a;
}

.badge-priority.warning {
  background-color: #372f1d;
  color: #eab308;
  border: 1px solid #a16207;
}

.badge-priority.normal {
  background-color: var(--bg-02);
  color: var(--text-02);
}

.progress-wrapper {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

/* Barre de progression Projet (Plus grande et épaisse) */
.project-progress-layout {
  margin-left: auto;
  width: 200px;
}

.progress-bar-container-large {
  flex-grow: 1;
  height: 10px;
  background-color: var(--bg-00);
  border-radius: 5px;
  overflow: hidden;
  border: var(--border-width) solid var(--bg-active);
}

.progress-bar-fill {
  height: 100%;
  background-color: var(--color-blue);
  border-radius: 5px;
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

/* Cercle de progression pour la Milestone */
.milestone-progress-circle-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  flex-shrink: 0;
}

.progress-circle {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.circle-bg {
  fill: none;
  stroke: var(--bg-00);
  stroke-width: 3.8;
}

.circle-fill {
  fill: none;
  stroke: var(--color-blue);
  stroke-width: 3.8;
  stroke-linecap: round;
  transition: stroke-dasharray 0.3s ease;
}

.circle-text {
  position: absolute;
  font-size: 0.6rem;
  font-family: monospace;
  font-weight: 600;
  color: var(--text-02);
}
</style>