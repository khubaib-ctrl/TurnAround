import { Component, signal, inject, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { ProjectService, RecentProject } from '../../services/project.service';
import { TauriService } from '../../services/tauri.service';
import { extractError } from '../../models/error.model';
import { open } from '@tauri-apps/plugin-dialog';

interface BackupEntry {
  name: string;
  original_path: string;
  backup_path: string;
  last_backup: string;
}

@Component({
  selector: 'app-project-setup',
  standalone: true,
  imports: [FormsModule],
  templateUrl: './project-setup.component.html',
  styleUrl: './project-setup.component.scss',
})
export class ProjectSetupComponent implements OnInit {
  private tauri = inject(TauriService);

  projectName = signal('');
  selectedPath = signal('');
  recentProjects = signal<RecentProject[]>([]);
  error = signal('');

  backups = signal<BackupEntry[]>([]);
  recoverTarget = signal('');
  recovering = signal(false);
  recoverError = signal('');

  constructor(
    private projectService: ProjectService,
    private router: Router,
  ) {
    this.recentProjects.set(this.projectService.getRecentProjects());
  }

  async ngOnInit() {
    await this.loadBackups();
  }

  async loadBackups() {
    try {
      const entries = await this.tauri.invoke<BackupEntry[]>('get_backup_registry');
      this.backups.set(entries);
    } catch (e: unknown) {
      console.warn('Failed to load backup registry:', extractError(e).message);
    }
  }

  async selectFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      this.selectedPath.set(selected as string);
      if (!this.projectName()) {
        const parts = (selected as string).split('/');
        this.projectName.set(parts[parts.length - 1] || 'Untitled');
      }
    }
  }

  async initProject() {
    if (!this.selectedPath() || !this.projectName()) return;
    this.error.set('');
    try {
      await this.projectService.initProject(this.selectedPath(), this.projectName());
      this.router.navigate(['/workspace']);
    } catch (e: unknown) {
      this.error.set(extractError(e).message);
    }
  }

  async openRecent(project: RecentProject) {
    this.error.set('');
    try {
      await this.projectService.openProject(project.path);
      this.router.navigate(['/workspace']);
    } catch (e: unknown) {
      this.error.set(extractError(e).message);
    }
  }

  async recoverFromBackup(entry: BackupEntry) {
    this.recoverError.set('');
    this.recovering.set(true);

    const target = await open({ directory: true, multiple: false });
    if (!target) {
      this.recovering.set(false);
      return;
    }

    try {
      await this.tauri.invoke('recover_project_from_backup', {
        originalPath: entry.original_path,
        targetPath: target as string,
      });
      await this.projectService.openProject(target as string);
      this.router.navigate(['/workspace']);
    } catch (e: unknown) {
      this.recoverError.set(extractError(e).message);
    } finally {
      this.recovering.set(false);
    }
  }

  formatBackupDate(isoDate: string): string {
    return new Date(isoDate).toLocaleString();
  }
}
