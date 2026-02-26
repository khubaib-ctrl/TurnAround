import { Component, OnInit, OnDestroy, signal, inject } from '@angular/core';
import { Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { Subscription } from 'rxjs';
import { GhostSidebarComponent } from '../../components/ghost-sidebar/ghost-sidebar.component';
import { GhostTimelineComponent } from '../../components/ghost-timeline/ghost-timeline.component';
import { TimeTravelSliderComponent } from '../../components/time-travel-slider/time-travel-slider.component';
import { CommitDialogComponent } from '../../components/commit-dialog/commit-dialog.component';
import { MilestoneButtonComponent } from '../../components/milestone-button/milestone-button.component';
import { FileTreeComponent } from '../../components/file-tree/file-tree.component';
import { ProjectService } from '../../services/project.service';
import { VcsService } from '../../services/vcs.service';
import { WatcherService, FileChangeEvent, ResolveProject } from '../../services/watcher.service';
import { TimelineService } from '../../services/timeline.service';
import { extractError } from '../../models/error.model';

@Component({
  selector: 'app-workspace',
  standalone: true,
  imports: [
    FormsModule,
    GhostSidebarComponent,
    GhostTimelineComponent,
    TimeTravelSliderComponent,
    CommitDialogComponent,
    MilestoneButtonComponent,
    FileTreeComponent,
  ],
  templateUrl: './workspace.component.html',
  styleUrl: './workspace.component.scss',
})
export class WorkspaceComponent implements OnInit, OnDestroy {
  private projectService = inject(ProjectService);
  private vcsService = inject(VcsService);
  private watcherService = inject(WatcherService);
  private timelineService = inject(TimelineService);
  private router = inject(Router);

  showCommitDialog = signal(false);
  commitDialogMilestone = signal(false);
  changedFiles = signal<string[]>([]);
  sidebarCollapsed = signal(false);
  sidebarTab = signal<'history' | 'files'>('history');

  showResolvePicker = signal(false);
  resolveProjects = signal<ResolveProject[]>([]);
  selectedResolveDb = signal('');
  linkedResolveName = signal<string | null>(null);
  loadingResolve = signal(false);

  private watcherSub?: Subscription;
  private changeCheckInterval?: ReturnType<typeof setInterval>;
  private lastDismissedAt = 0;
  private readonly CHANGE_CHECK_MS = 30000;
  private readonly DISMISS_COOLDOWN_MS = 30000;

  readonly project = this.projectService.project;
  readonly history = this.vcsService.history;
  readonly branches = this.vcsService.branches;
  readonly diff = this.timelineService.diff;
  readonly selectedCommit = this.vcsService.selectedCommit;

  async ngOnInit() {
    if (!this.projectService.isOpen()) {
      this.router.navigate(['/dashboard']);
      return;
    }

    await this.vcsService.getBranches();
    await this.vcsService.refreshHistory();

    await this.loadLinkedResolveName();

    try {
      await this.watcherService.startWatching();
    } catch (e: unknown) {
      console.warn('Failed to start watcher:', extractError(e).message);
    }

    this.watcherSub = this.watcherService.fileChanged$.subscribe((event: FileChangeEvent) => {
      const projectRoot = this.project()?.root_path;
      const relativePath = projectRoot && event.path.startsWith(projectRoot)
        ? event.path.slice(projectRoot.length).replace(/^\//, '')
        : event.path.split('/').pop() || event.path;

      this.changedFiles.update((files) => {
        const filtered = files.filter(f => f !== relativePath);
        return [...filtered, relativePath];
      });
    });

    this.changeCheckInterval = setInterval(() => this.checkForChangesAndShowDialog(), this.CHANGE_CHECK_MS);
  }

  ngOnDestroy() {
    this.watcherSub?.unsubscribe();
    if (this.changeCheckInterval) clearInterval(this.changeCheckInterval);
    this.watcherService.stopWatching();
  }

  private async checkForChangesAndShowDialog() {
    if (this.showCommitDialog()) return;
    if (Date.now() - this.lastDismissedAt < this.DISMISS_COOLDOWN_MS) return;
    try {
      const files = await this.vcsService.getChangedFiles();
      if (files.length > 0) {
        this.changedFiles.set(files);
      }
    } catch {
      // ignore (e.g. no project)
    }
  }

  async openCommitDialog(isMilestone = false) {
    this.commitDialogMilestone.set(isMilestone);
    try {
      const fromBackend = await this.vcsService.getChangedFiles();
      if (fromBackend.length > 0) {
        this.changedFiles.update((existing) => {
          const combined = new Set([...existing, ...fromBackend]);
          return [...combined];
        });
      }
    } catch (e: unknown) {
      console.warn('Could not refresh changed files:', extractError(e).message);
    }
    this.showCommitDialog.set(true);
  }

  async onCommitCreated() {
    this.showCommitDialog.set(false);
    this.changedFiles.set([]);
    await this.vcsService.refreshHistory();
  }

  onCommitDialogClose() {
    this.showCommitDialog.set(false);
    this.lastDismissedAt = Date.now();
  }

  toggleSidebar() {
    this.sidebarCollapsed.update((v) => !v);
  }

  enterCompareMode() {
    this.vcsService.enterCompareMode();
  }

  async openResolvePicker() {
    this.showResolvePicker.set(true);
    this.loadingResolve.set(true);
    try {
      const projects = await this.watcherService.listResolveProjects();
      this.resolveProjects.set(projects);
      const linked = await this.watcherService.getLinkedResolveProject();
      this.selectedResolveDb.set(linked || '');
    } catch {
      this.resolveProjects.set([]);
    } finally {
      this.loadingResolve.set(false);
    }
  }

  async saveResolveLink() {
    const dbPath = this.selectedResolveDb();
    try {
      if (dbPath) {
        await this.watcherService.linkResolveProject(dbPath);
      } else {
        await this.watcherService.unlinkResolveProject();
      }
      await this.loadLinkedResolveName();
      this.showResolvePicker.set(false);

      await this.watcherService.stopWatching();
      await this.watcherService.startWatching();
    } catch (e: unknown) {
      console.warn('Failed to update Resolve link:', extractError(e).message);
    }
  }

  private async loadLinkedResolveName() {
    try {
      const linked = await this.watcherService.getLinkedResolveProject();
      if (linked) {
        const parts = linked.split('/');
        const projectFolder = parts[parts.length - 2] || 'Resolve Project';
        this.linkedResolveName.set(projectFolder);
      } else {
        this.linkedResolveName.set(null);
      }
    } catch {
      this.linkedResolveName.set(null);
    }
  }

  async onBackToSetup() {
    await this.projectService.closeProject();
    this.router.navigate(['/dashboard']);
  }
}
