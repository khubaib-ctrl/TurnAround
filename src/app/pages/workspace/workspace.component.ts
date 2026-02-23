import { Component, OnInit, OnDestroy, signal, inject } from '@angular/core';
import { Router } from '@angular/router';
import { Subscription } from 'rxjs';
import { GhostSidebarComponent } from '../../components/ghost-sidebar/ghost-sidebar.component';
import { GhostTimelineComponent } from '../../components/ghost-timeline/ghost-timeline.component';
import { TimeTravelSliderComponent } from '../../components/time-travel-slider/time-travel-slider.component';
import { CommitDialogComponent } from '../../components/commit-dialog/commit-dialog.component';
import { MilestoneButtonComponent } from '../../components/milestone-button/milestone-button.component';
import { FileTreeComponent } from '../../components/file-tree/file-tree.component';
import { ProjectService } from '../../services/project.service';
import { VcsService } from '../../services/vcs.service';
import { WatcherService, FileChangeEvent } from '../../services/watcher.service';
import { TimelineService } from '../../services/timeline.service';
import { extractError } from '../../models/error.model';

@Component({
  selector: 'app-workspace',
  standalone: true,
  imports: [
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

  private watcherSub?: Subscription;

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

    try {
      await this.watcherService.startWatching();
    } catch (e: unknown) {
      console.warn('Failed to start watcher:', extractError(e).message);
    }

    this.watcherSub = this.watcherService.fileChanged$.subscribe((event: FileChangeEvent) => {
      this.changedFiles.update((files) => {
        if (!files.includes(event.path)) {
          return [...files, event.path];
        }
        return files;
      });
      this.showCommitDialog.set(true);
    });
  }

  ngOnDestroy() {
    this.watcherSub?.unsubscribe();
    this.watcherService.stopWatching();
  }

  openCommitDialog(isMilestone = false) {
    this.commitDialogMilestone.set(isMilestone);
    this.showCommitDialog.set(true);
  }

  async onCommitCreated() {
    this.showCommitDialog.set(false);
    this.changedFiles.set([]);
    await this.vcsService.refreshHistory();
  }

  onCommitDialogClose() {
    this.showCommitDialog.set(false);
  }

  toggleSidebar() {
    this.sidebarCollapsed.update((v) => !v);
  }

  enterCompareMode() {
    this.vcsService.enterCompareMode();
  }

  async onBackToSetup() {
    await this.projectService.closeProject();
    this.router.navigate(['/dashboard']);
  }
}
