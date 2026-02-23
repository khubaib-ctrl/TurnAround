import { Injectable, signal, computed } from '@angular/core';
import { TauriService } from './tauri.service';
import { Commit, CommitDetail, FileSnapshot } from '../models/commit.model';
import { Branch } from '../models/project.model';
import { extractError } from '../models/error.model';

export type FileDiffStatus = 'added' | 'removed' | 'modified' | 'unchanged';

export interface FileDiffEntry {
  file_path: string;
  status: FileDiffStatus;
  old_file: FileSnapshot | null;
  new_file: FileSnapshot | null;
  size_change: number;
}

export interface RestoreReport {
  total: number;
  restored_count: number;
  skipped_count: number;
  restored: string[];
  skipped: string[];
}

export interface ExportReport {
  commit_message: string;
  dest_path: string;
  total: number;
  exported_count: number;
  skipped_count: number;
  exported: string[];
  skipped: string[];
}

export interface CompareResult {
  commitA: Commit;
  commitB: Commit;
  files: FileDiffEntry[];
  summary: { added: number; removed: number; modified: number; unchanged: number };
}

@Injectable({ providedIn: 'root' })
export class VcsService {
  private _history = signal<Commit[]>([]);
  private _branches = signal<Branch[]>([]);
  private _selectedCommit = signal<Commit | null>(null);
  private _selectedDetail = signal<CommitDetail | null>(null);
  private _ghostCommit = signal<Commit | null>(null);
  private _compareMode = signal(false);
  private _compareResult = signal<CompareResult | null>(null);
  private _compareLoading = signal(false);
  private _compareError = signal('');

  readonly history = this._history.asReadonly();
  readonly branches = this._branches.asReadonly();
  readonly selectedCommit = this._selectedCommit.asReadonly();
  readonly selectedDetail = this._selectedDetail.asReadonly();
  readonly ghostCommit = this._ghostCommit.asReadonly();
  readonly compareMode = this._compareMode.asReadonly();
  readonly compareResult = this._compareResult.asReadonly();
  readonly compareLoading = this._compareLoading.asReadonly();
  readonly compareError = this._compareError.asReadonly();

  constructor(private tauri: TauriService) {}

  async createCommit(message: string, isMilestone: boolean): Promise<Commit> {
    const commit = await this.tauri.invoke<Commit>('create_commit', {
      message,
      isMilestone,
    });
    await this.refreshHistory();
    return commit;
  }

  async getHistory(branchId: string, limit = 100): Promise<Commit[]> {
    const commits = await this.tauri.invoke<Commit[]>('get_history', {
      branchId,
      limit,
    });
    this._history.set(commits);
    return commits;
  }

  async refreshHistory(): Promise<void> {
    const branches = await this.getBranches();
    const active = branches.find((b) => b.is_active);
    if (active) {
      await this.getHistory(active.id);
    }
  }

  async getCommitDetail(commitId: string): Promise<CommitDetail> {
    return this.tauri.invoke<CommitDetail>('get_commit_detail', { commitId });
  }

  async getBranches(): Promise<Branch[]> {
    const branches = await this.tauri.invoke<Branch[]>('get_branches');
    this._branches.set(branches);
    return branches;
  }

  async createBranch(name: string): Promise<Branch> {
    const branch = await this.tauri.invoke<Branch>('create_branch', { name });
    await this.getBranches();
    return branch;
  }

  async deleteCommit(commitId: string): Promise<void> {
    await this.tauri.invoke<void>('delete_commit', { commitId });
    this._selectedCommit.set(null);
    this._selectedDetail.set(null);
    await this.refreshHistory();
  }

  async deleteBranch(branchId: string): Promise<void> {
    await this.tauri.invoke<void>('delete_branch', { branchId });
    await this.getBranches();
  }

  async switchBranch(branchId: string): Promise<Branch> {
    const branch = await this.tauri.invoke<Branch>('switch_branch', { branchId });
    await this.getBranches();
    await this.refreshHistory();
    return branch;
  }

  async restoreCommit(commitId: string): Promise<RestoreReport> {
    return this.tauri.invoke<RestoreReport>('restore_commit', { commitId });
  }

  async exportCommit(commitId: string, destPath: string): Promise<ExportReport> {
    return this.tauri.invoke<ExportReport>('export_commit', { commitId, destPath });
  }

  async selectCommit(commit: Commit | null) {
    this._selectedCommit.set(commit);
    if (commit) {
      try {
        const detail = await this.getCommitDetail(commit.id);
        this._selectedDetail.set(detail);
      } catch (e: unknown) {
        console.warn('Failed to load commit detail:', extractError(e).message);
        this._selectedDetail.set(null);
      }
    } else {
      this._selectedDetail.set(null);
    }
  }

  setGhostCommit(commit: Commit | null) {
    this._ghostCommit.set(commit);
  }

  enterCompareMode() {
    this._compareMode.set(true);
    this._selectedCommit.set(null);
    this._selectedDetail.set(null);
    this._compareResult.set(null);
    this._compareError.set('');
  }

  exitCompareMode() {
    this._compareMode.set(false);
    this._compareResult.set(null);
    this._compareError.set('');
  }

  async compareCommits(commitAId: string, commitBId: string): Promise<void> {
    this._compareLoading.set(true);
    this._compareError.set('');
    this._compareResult.set(null);

    try {
      const [detailA, detailB] = await Promise.all([
        this.getCommitDetail(commitAId),
        this.getCommitDetail(commitBId),
      ]);

      const filesA = new Map(detailA.files.map((f) => [f.file_path, f]));
      const filesB = new Map(detailB.files.map((f) => [f.file_path, f]));
      const allPaths = new Set([...filesA.keys(), ...filesB.keys()]);

      const files: FileDiffEntry[] = [];
      const summary = { added: 0, removed: 0, modified: 0, unchanged: 0 };

      for (const path of allPaths) {
        const a = filesA.get(path) ?? null;
        const b = filesB.get(path) ?? null;

        if (a && !b) {
          summary.removed++;
          files.push({ file_path: path, status: 'removed', old_file: a, new_file: null, size_change: -a.file_size });
        } else if (!a && b) {
          summary.added++;
          files.push({ file_path: path, status: 'added', old_file: null, new_file: b, size_change: b.file_size });
        } else if (a && b) {
          if (a.content_hash === b.content_hash) {
            summary.unchanged++;
            files.push({ file_path: path, status: 'unchanged', old_file: a, new_file: b, size_change: 0 });
          } else {
            summary.modified++;
            files.push({ file_path: path, status: 'modified', old_file: a, new_file: b, size_change: b.file_size - a.file_size });
          }
        }
      }

      files.sort((x, y) => {
        const order: Record<string, number> = { added: 0, modified: 1, removed: 2, unchanged: 3 };
        return (order[x.status] ?? 4) - (order[y.status] ?? 4);
      });

      this._compareResult.set({
        commitA: detailA.commit,
        commitB: detailB.commit,
        files,
        summary,
      });
    } catch (e: unknown) {
      this._compareError.set(extractError(e).message);
    } finally {
      this._compareLoading.set(false);
    }
  }
}
