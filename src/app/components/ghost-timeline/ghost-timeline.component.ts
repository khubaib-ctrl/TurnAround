import { Component, computed, signal, inject } from '@angular/core';
import { DecimalPipe } from '@angular/common';
import { TimelineService } from '../../services/timeline.service';
import { VcsService, FileDiffEntry, RestoreReport } from '../../services/vcs.service';
import { ClipDiff, DiffStatus } from '../../models/timeline.model';
import { FileSnapshot } from '../../models/commit.model';
import { extractError } from '../../models/error.model';

@Component({
  selector: 'app-ghost-timeline',
  standalone: true,
  imports: [DecimalPipe],
  templateUrl: './ghost-timeline.component.html',
  styleUrl: './ghost-timeline.component.scss',
})
export class GhostTimelineComponent {
  private timelineService = inject(TimelineService);
  private vcsService = inject(VcsService);

  readonly diff = this.timelineService.diff;
  readonly loading = this.timelineService.loading;
  readonly ghostCommit = this.vcsService.ghostCommit;
  readonly selectedCommit = this.vcsService.selectedCommit;
  readonly selectedDetail = this.vcsService.selectedDetail;
  readonly compareMode = this.vcsService.compareMode;
  readonly compareResult = this.vcsService.compareResult;
  readonly compareLoading = this.vcsService.compareLoading;
  readonly compareError = this.vcsService.compareError;
  readonly history = this.vcsService.history;
  zoom = signal(1);

  compareCommitA = signal('');
  compareCommitB = signal('');

  restoreConfirm = signal(false);
  restoring = signal(false);
  restoreResult = signal<RestoreReport | null>(null);
  restoreError = signal('');

  readonly hasDiff = computed(() => this.diff() !== null);
  readonly hasDetail = computed(() => this.selectedDetail() !== null);
  readonly hasCompareResult = computed(() => this.compareResult() !== null);
  readonly canCompare = computed(() => {
    const a = this.compareCommitA();
    const b = this.compareCommitB();
    return a && b && a !== b;
  });

  onSelectA(event: Event) {
    this.compareCommitA.set((event.target as HTMLSelectElement).value);
  }

  onSelectB(event: Event) {
    this.compareCommitB.set((event.target as HTMLSelectElement).value);
  }

  async runCompare() {
    if (!this.canCompare()) return;
    await this.vcsService.compareCommits(this.compareCommitA(), this.compareCommitB());
  }

  exitCompare() {
    this.vcsService.exitCompareMode();
    this.compareCommitA.set('');
    this.compareCommitB.set('');
  }

  getDiffStatusClass(status: string): string {
    return `file-${status}`;
  }

  formatSizeChange(bytes: number): string {
    const sign = bytes >= 0 ? '+' : '';
    if (Math.abs(bytes) < 1024) return `${sign}${bytes} B`;
    if (Math.abs(bytes) < 1024 * 1024) return `${sign}${(bytes / 1024).toFixed(1)} KB`;
    return `${sign}${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  zoomIn() {
    this.zoom.update((z) => Math.min(z * 1.2, 5));
  }

  zoomOut() {
    this.zoom.update((z) => Math.max(z / 1.2, 0.3));
  }

  resetZoom() {
    this.zoom.set(1);
  }

  getClipWidth(clip: ClipDiff): number {
    const range = clip.new_range || clip.old_range;
    if (!range) return 120;
    const seconds = range.duration.rate > 0 ? range.duration.value / range.duration.rate : 2;
    return Math.max(seconds * 80 * this.zoom(), 40);
  }

  getClipOffset(clip: ClipDiff): number {
    const range = clip.new_range || clip.old_range;
    if (!range) return 0;
    const seconds = range.start.rate > 0 ? range.start.value / range.start.rate : 0;
    return seconds * 80 * this.zoom();
  }

  getStatusClass(status: DiffStatus): string {
    switch (status) {
      case 'Added':
        return 'clip-added';
      case 'Removed':
        return 'clip-removed';
      case 'Modified':
        return 'clip-modified';
      default:
        return 'clip-unchanged';
    }
  }

  getStatusLabel(status: DiffStatus): string {
    switch (status) {
      case 'Added':
        return 'NEW';
      case 'Removed':
        return 'DEL';
      case 'Modified':
        return 'MOD';
      default:
        return '';
    }
  }

  formatDuration(clip: ClipDiff): string {
    const range = clip.new_range || clip.old_range;
    if (!range) return '';
    const sec = range.duration.rate > 0 ? range.duration.value / range.duration.rate : 0;
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    const f = Math.floor((sec % 1) * (range.duration.rate || 24));
    return m > 0
      ? `${m}:${s.toString().padStart(2, '0')}:${f.toString().padStart(2, '0')}`
      : `${s}:${f.toString().padStart(2, '0')}`;
  }

  getTrackIcon(kind: string): string {
    return kind === 'Audio' ? 'A' : 'V';
  }

  getFileName(filePath: string): string {
    const parts = filePath.split('/');
    return parts[parts.length - 1] || filePath;
  }

  getFileDir(filePath: string): string {
    const parts = filePath.split('/');
    if (parts.length <= 1) return '';
    return parts.slice(0, -1).join('/') + '/';
  }

  formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  getFileIcon(fileType: string): string {
    switch (fileType.toLowerCase()) {
      case 'video': return 'V';
      case 'audio': return 'A';
      case 'image': return 'I';
      case 'project': return 'P';
      default: return 'F';
    }
  }

  formatCommitTime(isoDate: string): string {
    return new Date(isoDate).toLocaleString();
  }

  promptRestore() {
    this.restoreConfirm.set(true);
    this.restoreResult.set(null);
    this.restoreError.set('');
  }

  cancelRestore() {
    this.restoreConfirm.set(false);
  }

  async confirmRestore() {
    const detail = this.selectedDetail();
    if (!detail) return;

    this.restoring.set(true);
    this.restoreError.set('');
    try {
      const report = await this.vcsService.restoreCommit(detail.commit.id);
      this.restoreResult.set(report);
      this.restoreConfirm.set(false);
    } catch (e: unknown) {
      this.restoreError.set(extractError(e).message);
    } finally {
      this.restoring.set(false);
    }
  }

  dismissRestoreResult() {
    this.restoreResult.set(null);
  }
}
