import { Injectable, signal } from '@angular/core';
import { TauriService } from './tauri.service';
import { TimelineDiff } from '../models/timeline.model';

@Injectable({ providedIn: 'root' })
export class TimelineService {
  private _diff = signal<TimelineDiff | null>(null);
  private _loading = signal(false);

  readonly diff = this._diff.asReadonly();
  readonly loading = this._loading.asReadonly();

  constructor(private tauri: TauriService) {}

  async getTimelineDiff(commitA: string, commitB: string): Promise<TimelineDiff> {
    this._loading.set(true);
    try {
      const diff = await this.tauri.invoke<TimelineDiff>('get_timeline_diff', {
        commitA,
        commitB,
      });
      this._diff.set(diff);
      return diff;
    } finally {
      this._loading.set(false);
    }
  }

  clearDiff() {
    this._diff.set(null);
  }
}
