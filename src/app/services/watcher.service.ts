import { Injectable, signal } from '@angular/core';
import { TauriService } from './tauri.service';
import { Subject } from 'rxjs';
import type { UnlistenFn } from '@tauri-apps/api/event';

export interface FileChangeEvent {
  path: string;
  kind: string;
}

export interface ResolveProject {
  name: string;
  db_path: string;
}

@Injectable({ providedIn: 'root' })
export class WatcherService {
  private _watching = signal(false);
  private _unlisten: UnlistenFn | null = null;
  private _linkedResolve = signal<string | null>(null);

  readonly watching = this._watching.asReadonly();
  readonly linkedResolve = this._linkedResolve.asReadonly();
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

  async listResolveProjects(): Promise<ResolveProject[]> {
    return this.tauri.invoke<ResolveProject[]>('list_resolve_projects');
  }

  async linkResolveProject(dbPath: string): Promise<void> {
    await this.tauri.invoke('link_resolve_project', { dbPath });
    this._linkedResolve.set(dbPath);
  }

  async unlinkResolveProject(): Promise<void> {
    await this.tauri.invoke('unlink_resolve_project');
    this._linkedResolve.set(null);
  }

  async getLinkedResolveProject(): Promise<string | null> {
    const path = await this.tauri.invoke<string | null>('get_linked_resolve_project');
    this._linkedResolve.set(path);
    return path;
  }
}
