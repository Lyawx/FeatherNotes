export type TabId = 'ai' | 'logbook' | 'lifewiki' | 'tasks' | 'health' | 'settings';

export interface Tab {
  id: TabId;
  label: string;
  icon: string;
}