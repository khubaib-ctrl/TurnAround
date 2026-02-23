import { Injectable, signal } from '@angular/core';
import { TauriService } from './tauri.service';
import { Subject } from 'rxjs';
import type { UnlistenFn } from '@tauri-apps/api/event';

export interface FileChangeEvent {
  path: string;
  kind: string;
}

@Injectable({ providedIn: 'root' })
export class WatcherService {
  private _watching = signal(false);
  private _unlisten: UnlistenFn | null = null;

  readonly watching = this._watching.asReadonly();
  readonly fileChanged$ = new Subject<FileChangeEvent>();

  constructor(private tauri: TauriService) {}

  async startWatching(): Promise<void> {
    await this.tauri.invoke('start_watching');
    this._watching.set(true);

    this._unlisten = await this.tauri.listen<FileChangeEvent>(
      'editgit://file-changed',
      (event) => this.fileChanged$.next(event),
    );
  }

  async stopWatching(): Promise<void> {
    await this.tauri.invoke('stop_watching');
    this._watching.set(false);
    if (this._unlisten) {
      this._unlisten();
      this._unlisten = null;
    }
  }
}
