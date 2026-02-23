import { Component, signal, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { VcsService } from '../../services/vcs.service';
import { Commit } from '../../models/commit.model';
import { Branch } from '../../models/project.model';
import { extractError } from '../../models/error.model';
import { open as dialogOpen } from '@tauri-apps/plugin-dialog';

@Component({
  selector: 'app-ghost-sidebar',
  standalone: true,
  imports: [FormsModule],
  templateUrl: './ghost-sidebar.component.html',
  styleUrl: './ghost-sidebar.component.scss',
})
export class GhostSidebarComponent {
  private vcsService = inject(VcsService);

  showNewBranch = signal(false);
  newBranchName = signal('');
  branchError = signal('');
  creatingBranch = signal(false);
  confirmingDelete = signal<string | null>(null);
  confirmingCommitDelete = signal<string | null>(null);
  commitDeleteError = signal('');
  exportStatus = signal('');
  exportSuccess = signal(false);

  readonly history = this.vcsService.history;
  readonly branches = this.vcsService.branches;
  readonly selectedCommit = this.vcsService.selectedCommit;

  getActiveBranch(): Branch | undefined {
    return this.branches().find((b) => b.is_active);
  }

  async onBranchChange(event: Event) {
    const branchId = (event.target as HTMLSelectElement).value;
    await this.vcsService.switchBranch(branchId);
  }

  onCommitClick(commit: Commit) {
    this.vcsService.selectCommit(commit);
  }

  toggleNewBranch() {
    this.confirmingDelete.set(null);
    this.showNewBranch.update((v) => !v);
    this.newBranchName.set('');
    this.branchError.set('');
  }

  async createBranch() {
    const name = this.newBranchName().trim();
    if (!name) return;

    this.branchError.set('');
    this.creatingBranch.set(true);
    try {
      const branch = await this.vcsService.createBranch(name);
      await this.vcsService.switchBranch(branch.id);
      this.showNewBranch.set(false);
      this.newBranchName.set('');
    } catch (e: unknown) {
      this.branchError.set(extractError(e).message);
    } finally {
      this.creatingBranch.set(false);
    }
  }

  promptDeleteBranch(branchId: string) {
    this.showNewBranch.set(false);
    this.confirmingDelete.set(branchId);
    this.branchError.set('');
  }

  cancelDelete() {
    this.confirmingDelete.set(null);
  }

  async confirmDeleteBranch() {
    const branchId = this.confirmingDelete();
    if (!branchId) return;
    this.branchError.set('');
    try {
      await this.vcsService.deleteBranch(branchId);
      this.confirmingDelete.set(null);
    } catch (e: unknown) {
      this.branchError.set(extractError(e).message);
    }
  }

  canDeleteBranch(branch: Branch): boolean {
    return !branch.is_active && this.branches().length > 1;
  }

  isLatestCommit(commit: Commit): boolean {
    const h = this.history();
    return h.length > 0 && h[0].id === commit.id;
  }

  promptDeleteCommit(commitId: string, event: Event) {
    event.stopPropagation();
    this.confirmingCommitDelete.set(commitId);
    this.commitDeleteError.set('');
  }

  cancelCommitDelete() {
    this.confirmingCommitDelete.set(null);
    this.commitDeleteError.set('');
  }

  async confirmDeleteCommit() {
    const commitId = this.confirmingCommitDelete();
    if (!commitId) return;
    this.commitDeleteError.set('');
    try {
      await this.vcsService.deleteCommit(commitId);
      this.confirmingCommitDelete.set(null);
    } catch (e: unknown) {
      this.commitDeleteError.set(extractError(e).message);
    }
  }

  async downloadCommit(commit: Commit, event: Event) {
    event.stopPropagation();

    const selectedDir = await dialogOpen({
      directory: true,
      multiple: false,
      title: 'Choose download folder',
    });

    if (!selectedDir) return;

    this.exportStatus.set('Exporting...');
    this.exportSuccess.set(false);

    try {
      const report = await this.vcsService.exportCommit(commit.id, selectedDir as string);
      this.exportSuccess.set(true);
      this.exportStatus.set(
        `Exported ${report.exported_count}/${report.total} files to ${report.dest_path}`
      );
    } catch (e: unknown) {
      this.exportSuccess.set(false);
      this.exportStatus.set(extractError(e).message);
    }
  }

  formatTime(isoDate: string): string {
    const d = new Date(isoDate);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMin = Math.floor(diffMs / 60000);

    if (diffMin < 1) return 'Just now';
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHrs = Math.floor(diffMin / 60);
    if (diffHrs < 24) return `${diffHrs}h ago`;
    const diffDays = Math.floor(diffHrs / 24);
    if (diffDays < 7) return `${diffDays}d ago`;
    return d.toLocaleDateString();
  }
}
