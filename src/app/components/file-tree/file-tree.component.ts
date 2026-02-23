import { Component, signal, inject, OnInit } from '@angular/core';
import { NgTemplateOutlet } from '@angular/common';
import { TauriService } from '../../services/tauri.service';
import { extractError } from '../../models/error.model';

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
  children: FileNode[] | null;
}

@Component({
  selector: 'app-file-tree',
  standalone: true,
  imports: [NgTemplateOutlet],
  templateUrl: './file-tree.component.html',
  styleUrl: './file-tree.component.scss',
})
export class FileTreeComponent implements OnInit {
  private tauri = inject(TauriService);

  tree = signal<FileNode | null>(null);
  loading = signal(false);
  error = signal('');
  expanded = signal<Set<string>>(new Set());

  async ngOnInit() {
    await this.loadTree();
  }

  async loadTree() {
    this.loading.set(true);
    this.error.set('');
    try {
      const tree = await this.tauri.invoke<FileNode>('get_project_tree');
      this.tree.set(tree);
      if (tree.children) {
        this.expanded.update((s) => {
          const ns = new Set(s);
          ns.add('');
          return ns;
        });
      }
    } catch (e: unknown) {
      this.error.set(extractError(e).message);
    } finally {
      this.loading.set(false);
    }
  }

  toggleDir(path: string) {
    this.expanded.update((s) => {
      const ns = new Set(s);
      if (ns.has(path)) {
        ns.delete(path);
      } else {
        ns.add(path);
      }
      return ns;
    });
  }

  isExpanded(path: string): boolean {
    return this.expanded().has(path);
  }

  getFileIcon(node: FileNode): string {
    if (node.is_dir) return 'folder';
    const ext = node.name.split('.').pop()?.toLowerCase() || '';
    switch (ext) {
      case 'mp4': case 'mov': case 'avi': case 'mkv': case 'mxf': case 'webm':
        return 'video';
      case 'wav': case 'mp3': case 'aac': case 'flac': case 'ogg': case 'm4a': case 'aiff':
        return 'audio';
      case 'png': case 'jpg': case 'jpeg': case 'tif': case 'tiff': case 'exr': case 'bmp': case 'gif': case 'webp': case 'psd': case 'svg':
        return 'image';
      case 'drp': case 'prproj': case 'fcpxml': case 'otio': case 'aaf':
        return 'project';
      case 'srt': case 'ass':
        return 'subtitle';
      case 'xml': case 'json':
        return 'config';
      default:
        return 'file';
    }
  }

  formatSize(bytes: number | null): string {
    if (bytes === null) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }
}
