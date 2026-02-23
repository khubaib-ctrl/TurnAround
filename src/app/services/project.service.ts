import { Injectable, signal, computed } from '@angular/core';
import { TauriService } from './tauri.service';
import { Project, Branch } from '../models/project.model';

const RECENT_PROJECTS_KEY = 'editgit_recent_projects';

export interface RecentProject {
  name: string;
  path: string;
  lastOpened: string;
}

@Injectable({ providedIn: 'root' })
export class ProjectService {
  private _project = signal<Project | null>(null);
  private _loading = signal(false);

  readonly project = this._project.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly isOpen = computed(() => this._project() !== null);

  constructor(private tauri: TauriService) {}

  async initProject(path: string, name: string): Promise<Project> {
    this._loading.set(true);
    try {
      const project = await this.tauri.invoke<Project>('init_project', { path, name });
      this._project.set(project);
      this.addToRecent({ name, path, lastOpened: new Date().toISOString() });
      return project;
    } finally {
      this._loading.set(false);
    }
  }

  async openProject(path: string): Promise<Project> {
    this._loading.set(true);
    try {
      const project = await this.tauri.invoke<Project>('open_project', { path });
      this._project.set(project);
      this.addToRecent({ name: project.name, path, lastOpened: new Date().toISOString() });
      return project;
    } finally {
      this._loading.set(false);
    }
  }

  async closeProject(): Promise<void> {
    await this.tauri.invoke('close_project');
    this._project.set(null);
  }

  async refreshProject(): Promise<void> {
    const info = await this.tauri.invoke<Project | null>('get_project_info');
    this._project.set(info);
  }

  getRecentProjects(): RecentProject[] {
    try {
      const raw = localStorage.getItem(RECENT_PROJECTS_KEY);
      return raw ? JSON.parse(raw) : [];
    } catch {
      return [];
    }
  }

  private addToRecent(entry: RecentProject) {
    let recents = this.getRecentProjects().filter((r) => r.path !== entry.path);
    recents.unshift(entry);
    recents = recents.slice(0, 10);
    localStorage.setItem(RECENT_PROJECTS_KEY, JSON.stringify(recents));
  }
}
