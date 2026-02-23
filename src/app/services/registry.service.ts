import { Injectable, signal } from '@angular/core';
import { TauriService } from './tauri.service';
import { UserProfile, ProjectEntry, ProjectStats } from '../models/project.model';

@Injectable({ providedIn: 'root' })
export class RegistryService {
  private _profile = signal<UserProfile | null>(null);
  private _projects = signal<ProjectEntry[]>([]);
  private _loading = signal(false);

  readonly profile = this._profile.asReadonly();
  readonly projects = this._projects.asReadonly();
  readonly loading = this._loading.asReadonly();

  constructor(private tauri: TauriService) {}

  async loadProfile(): Promise<UserProfile | null> {
    const profile = await this.tauri.invoke<UserProfile | null>('get_user_profile');
    this._profile.set(profile);
    return profile;
  }

  async saveProfile(displayName: string, email: string): Promise<UserProfile> {
    const profile = await this.tauri.invoke<UserProfile>('save_user_profile', {
      displayName,
      email,
    });
    this._profile.set(profile);
    return profile;
  }

  async loadProjects(): Promise<ProjectEntry[]> {
    this._loading.set(true);
    try {
      const projects = await this.tauri.invoke<ProjectEntry[]>('list_projects');
      this._projects.set(projects);
      return projects;
    } finally {
      this._loading.set(false);
    }
  }

  async renameProject(projectId: string, newName: string): Promise<void> {
    await this.tauri.invoke<void>('rename_project', { projectId, newName });
    await this.loadProjects();
  }

  async updateDescription(projectId: string, description: string): Promise<void> {
    await this.tauri.invoke<void>('update_project_description', { projectId, description });
    await this.loadProjects();
  }

  async archiveProject(projectId: string): Promise<void> {
    await this.tauri.invoke<void>('archive_project', { projectId });
    await this.loadProjects();
  }

  async unarchiveProject(projectId: string): Promise<void> {
    await this.tauri.invoke<void>('unarchive_project', { projectId });
    await this.loadProjects();
  }

  async deleteProject(projectId: string, deleteData: boolean): Promise<void> {
    await this.tauri.invoke<void>('delete_project_from_registry', { projectId, deleteData });
    await this.loadProjects();
  }

  async getProjectStats(projectId: string): Promise<ProjectStats> {
    return this.tauri.invoke<ProjectStats>('get_project_stats_live', { projectId });
  }
}
