import { Component, signal, computed, inject, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { RegistryService } from '../../services/registry.service';
import { ProjectService } from '../../services/project.service';
import { WatcherService, ResolveProject } from '../../services/watcher.service';
import { ProjectEntry, UserProfile } from '../../models/project.model';
import { extractError } from '../../models/error.model';
import { open } from '@tauri-apps/plugin-dialog';
import { homeDir } from '@tauri-apps/api/path';

type FilterTab = 'all' | 'active' | 'archived';
type SortMode = 'recent' | 'name' | 'size' | 'commits';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [FormsModule],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss',
})
export class DashboardComponent implements OnInit {
  private registryService = inject(RegistryService);
  private projectService = inject(ProjectService);
  private watcherService = inject(WatcherService);
  private router = inject(Router);

  searchQuery = signal('');
  filterTab = signal<FilterTab>('all');
  sortMode = signal<SortMode>('recent');

  showProfileDialog = signal(false);
  profileName = signal('');
  profileEmail = signal('');
  savingProfile = signal(false);

  showNewProjectDialog = signal(false);
  newProjectName = signal('');
  newProjectPath = signal('');
  creatingProject = signal(false);
  newProjectError = signal('');

  resolveProjects = signal<ResolveProject[]>([]);
  selectedResolveDb = signal('');
  loadingResolve = signal(false);
  copiedPath = signal(false);

  renamingProject = signal<string | null>(null);
  renameValue = signal('');

  confirmDeleteId = signal<string | null>(null);
  deleteDataToo = signal(false);

  error = signal('');

  readonly profile = this.registryService.profile;
  readonly projects = this.registryService.projects;
  readonly loading = this.registryService.loading;

  readonly filteredProjects = computed(() => {
    let items = this.projects();
    const tab = this.filterTab();
    const q = this.searchQuery().toLowerCase().trim();

    if (tab === 'active') items = items.filter((p) => !p.is_archived);
    else if (tab === 'archived') items = items.filter((p) => p.is_archived);

    if (q) {
      items = items.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          p.description.toLowerCase().includes(q) ||
          p.root_path.toLowerCase().includes(q) ||
          p.tags.toLowerCase().includes(q),
      );
    }

    const mode = this.sortMode();
    return [...items].sort((a, b) => {
      if (mode === 'name') return a.name.localeCompare(b.name);
      if (mode === 'size') return b.disk_usage_bytes - a.disk_usage_bytes;
      if (mode === 'commits') return b.commit_count - a.commit_count;
      return new Date(b.last_opened_at).getTime() - new Date(a.last_opened_at).getTime();
    });
  });

  readonly activeCount = computed(() => this.projects().filter((p) => !p.is_archived).length);
  readonly archivedCount = computed(() => this.projects().filter((p) => p.is_archived).length);
  readonly totalDiskUsage = computed(() =>
    this.projects().reduce((acc, p) => acc + p.disk_usage_bytes, 0),
  );

  async ngOnInit() {
    await Promise.all([
      this.registryService.loadProfile(),
      this.registryService.loadProjects(),
    ]);

    if (!this.profile()) {
      this.showProfileDialog.set(true);
    }
  }

  // ── Profile ──

  openProfileDialog() {
    const p = this.profile();
    this.profileName.set(p?.display_name || '');
    this.profileEmail.set(p?.email || '');
    this.showProfileDialog.set(true);
  }

  async saveProfile() {
    const name = this.profileName().trim();
    if (!name) return;
    this.savingProfile.set(true);
    try {
      await this.registryService.saveProfile(name, this.profileEmail().trim());
      this.showProfileDialog.set(false);
    } finally {
      this.savingProfile.set(false);
    }
  }

  // ── New Project ──

  private homeDirPath = '';
  showAdvancedNew = signal(false);

  async openNewProjectDialog() {
    this.newProjectName.set('');
    this.newProjectPath.set('');
    this.newProjectError.set('');
    this.selectedResolveDb.set('');
    this.showAdvancedNew.set(false);
    this.showNewProjectDialog.set(true);
    this.loadingResolve.set(true);
    try {
      const [projects, home] = await Promise.all([
        this.watcherService.listResolveProjects(),
        this.homeDirPath ? Promise.resolve(this.homeDirPath) : homeDir(),
      ]);
      this.homeDirPath = home;
      this.resolveProjects.set(projects);
    } catch {
      this.resolveProjects.set([]);
    } finally {
      this.loadingResolve.set(false);
    }
  }

  async onResolveSelect(dbPath: string) {
    this.selectedResolveDb.set(dbPath);
    if (!dbPath) return;
    const rp = this.resolveProjects().find((p) => p.db_path === dbPath);
    if (rp) {
      this.newProjectName.set(rp.name);
      const base = this.homeDirPath
        ? `${this.homeDirPath}Documents/Turn Around`
        : '/tmp/Turn Around';
      this.newProjectPath.set(`${base}/${rp.name}`);
    }
  }

  async pickFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      this.newProjectPath.set(selected as string);
      if (!this.newProjectName()) {
        const parts = (selected as string).split('/');
        this.newProjectName.set(parts[parts.length - 1] || 'Untitled');
      }
      this.autoMatchResolve();
    }
  }

  onProjectNameChange(name: string) {
    this.newProjectName.set(name);
    this.autoMatchResolve();
    const base = this.homeDirPath
      ? `${this.homeDirPath}Documents/Turn Around`
      : '/tmp/Turn Around';
    this.newProjectPath.set(`${base}/${name.trim()}`);
  }

  private autoMatchResolve() {
    const name = this.newProjectName().trim().toLowerCase();
    if (!name || this.resolveProjects().length === 0) return;
    const match = this.resolveProjects().find(
      (rp) => rp.name.toLowerCase() === name,
    );
    if (match) {
      this.selectedResolveDb.set(match.db_path);
    }
  }

  copyPath() {
    navigator.clipboard.writeText(this.newProjectPath());
    this.copiedPath.set(true);
    setTimeout(() => this.copiedPath.set(false), 2000);
  }

  async createProject() {
    if (!this.newProjectPath() || !this.newProjectName()) return;
    this.creatingProject.set(true);
    this.newProjectError.set('');
    try {
      await this.projectService.initProject(this.newProjectPath(), this.newProjectName());
      if (this.selectedResolveDb()) {
        await this.watcherService.linkResolveProject(this.selectedResolveDb());
      }
      await this.projectService.closeProject();
      await this.registryService.loadProjects();
      this.showNewProjectDialog.set(false);
    } catch (e: unknown) {
      this.newProjectError.set(extractError(e).message);
    } finally {
      this.creatingProject.set(false);
    }
  }

  // ── Open Project ──

  async openProject(project: ProjectEntry) {
    this.error.set('');
    try {
      await this.projectService.openProject(project.root_path);
      this.router.navigate(['/workspace']);
    } catch (e: unknown) {
      this.error.set(extractError(e).message);
    }
  }

  // ── Rename ──

  startRename(project: ProjectEntry, event: Event) {
    event.stopPropagation();
    this.renamingProject.set(project.id);
    this.renameValue.set(project.name);
  }

  async confirmRename(projectId: string) {
    const name = this.renameValue().trim();
    if (!name) return;
    try {
      await this.registryService.renameProject(projectId, name);
    } catch (e: unknown) {
      console.warn('Failed to rename project:', extractError(e).message);
    }
    this.renamingProject.set(null);
  }

  cancelRename() {
    this.renamingProject.set(null);
  }

  // ── Archive / Unarchive ──

  async toggleArchive(project: ProjectEntry, event: Event) {
    event.stopPropagation();
    if (project.is_archived) {
      await this.registryService.unarchiveProject(project.id);
    } else {
      await this.registryService.archiveProject(project.id);
    }
  }

  // ── Delete ──

  promptDelete(project: ProjectEntry, event: Event) {
    event.stopPropagation();
    this.confirmDeleteId.set(project.id);
    this.deleteDataToo.set(false);
  }

  async confirmDelete() {
    const id = this.confirmDeleteId();
    if (!id) return;
    await this.registryService.deleteProject(id, this.deleteDataToo());
    this.confirmDeleteId.set(null);
  }

  cancelDelete() {
    this.confirmDeleteId.set(null);
  }

  // ── Open existing project folder ──

  async openExistingFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;
    this.error.set('');
    try {
      await this.projectService.openProject(selected as string);
      this.router.navigate(['/workspace']);
    } catch (e: unknown) {
      this.error.set(extractError(e).message);
    }
  }

  // ── Helpers ──

  formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
  }

  formatRelativeTime(iso: string): string {
    const d = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMin = Math.floor(diffMs / 60000);
    if (diffMin < 1) return 'Just now';
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHrs = Math.floor(diffMin / 60);
    if (diffHrs < 24) return `${diffHrs}h ago`;
    const diffDays = Math.floor(diffHrs / 24);
    if (diffDays < 30) return `${diffDays}d ago`;
    return d.toLocaleDateString();
  }

  getInitials(name: string): string {
    return name
      .split(' ')
      .map((w) => w[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  }

  truncatePath(path: string, max = 45): string {
    if (path.length <= max) return path;
    return '...' + path.slice(path.length - max);
  }
}
