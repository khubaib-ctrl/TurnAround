import { Component, input, output, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { VcsService } from '../../services/vcs.service';
import { extractError } from '../../models/error.model';

@Component({
  selector: 'app-commit-dialog',
  standalone: true,
  imports: [FormsModule],
  templateUrl: './commit-dialog.component.html',
  styleUrl: './commit-dialog.component.scss',
})
export class CommitDialogComponent {
  isMilestone = input(false);
  changedFiles = input<string[]>([]);
  committed = output<void>();
  closed = output<void>();

  message = signal('');
  markAsMilestone = signal(false);
  saving = signal(false);
  error = signal('');

  constructor(private vcsService: VcsService) {}

  ngOnInit() {
    this.markAsMilestone.set(this.isMilestone());
  }

  async commit() {
    const msg = this.message().trim();
    if (!msg) {
      this.error.set('Please enter a description');
      return;
    }

    this.saving.set(true);
    this.error.set('');

    try {
      await this.vcsService.createCommit(msg, this.markAsMilestone());
      this.committed.emit();
    } catch (e: unknown) {
      this.error.set(extractError(e).message);
    } finally {
      this.saving.set(false);
    }
  }

  close() {
    this.closed.emit();
  }

  onOverlayClick(event: MouseEvent) {
    if ((event.target as HTMLElement).classList.contains('dialog-overlay')) {
      this.close();
    }
  }

  getFileName(path: string): string {
    return path.split('/').pop() || path;
  }
}
