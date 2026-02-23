export interface Project {
  id: string;
  name: string;
  root_path: string;
  created_at: string;
  active_branch?: Branch;
}

export interface Branch {
  id: string;
  project_id: string;
  name: string;
  head_commit_id: string | null;
  is_active: boolean;
}

export interface UserProfile {
  id: string;
  display_name: string;
  email: string;
  avatar_path: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProjectEntry {
  id: string;
  name: string;
  description: string;
  root_path: string;
  tags: string;
  is_archived: boolean;
  last_opened_at: string;
  created_at: string;
  disk_usage_bytes: number;
  commit_count: number;
  branch_count: number;
}

export interface ProjectStats {
  commit_count: number;
  branch_count: number;
  disk_usage_bytes: number;
}
