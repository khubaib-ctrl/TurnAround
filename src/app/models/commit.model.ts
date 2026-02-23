export interface Commit {
  id: string;
  project_id: string;
  branch_id: string;
  parent_id: string | null;
  message: string;
  is_milestone: boolean;
  created_at: string;
}

export interface FileSnapshot {
  id: string;
  commit_id: string;
  file_path: string;
  content_hash: string;
  file_size: number;
  file_type: string;
}

export interface CommitDetail {
  commit: Commit;
  files: FileSnapshot[];
}
