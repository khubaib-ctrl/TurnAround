import { Component, computed, signal, inject } from '@angular/core';
import { VcsService } from '../../services/vcs.service';
import { TimelineService } from '../../services/timeline.service';
import { Commit } from '../../models/commit.model';
import { extractError } from '../../models/error.model';

@Component({
  selector: 'app-time-travel-slider',
  standalone: true,
  templateUrl: './time-travel-slider.component.html',
  styleUrl: './time-travel-slider.component.scss',
})
export class TimeTravelSliderComponent {
  private vcsService = inject(VcsService);
  private timelineService = inject(TimelineService);

  readonly history = this.vcsService.history;
  readonly ghostCommit = this.vcsService.ghostCommit;
  hoveredIndex = signal<number | null>(null);

  readonly max = computed(() => Math.max(this.history().length - 1, 0));
  readonly currentIndex = computed(() => {
    const ghost = this.ghostCommit();
    if (!ghost) return this.max();
    const idx = this.history().findIndex((c) => c.id === ghost.id);
    return idx >= 0 ? idx : this.max();
  });

  readonly hoveredCommit = computed(() => {
    const idx = this.hoveredIndex();
    if (idx === null) return null;
    return this.history()[idx] ?? null;
  });

  onSliderChange(event: Event) {
    const index = Number((event.target as HTMLInputElement).value);
    const commits = this.history();
    if (index >= 0 && index < commits.length) {
      const commit = commits[index];
      this.vcsService.setGhostCommit(commit);

      const latest = commits[0];
      if (latest && latest.id !== commit.id) {
        this.timelineService.getTimelineDiff(commit.id, latest.id).catch((e: unknown) => {
          console.debug('Timeline diff unavailable:', extractError(e).message);
        });
      } else {
        this.timelineService.clearDiff();
      }
    }
  }

  onSliderHover(event: MouseEvent) {
    const slider = event.target as HTMLInputElement;
    const rect = slider.getBoundingClientRect();
    const pct = (event.clientX - rect.left) / rect.width;
    const idx = Math.round(pct * this.max());
    this.hoveredIndex.set(Math.max(0, Math.min(idx, this.max())));
  }

  onSliderLeave() {
    this.hoveredIndex.set(null);
  }

  formatTime(isoDate: string): string {
    return new Date(isoDate).toLocaleString();
  }
}
