export interface SkillListItem {
  id: string;
  folderName: string;
  description: string | null;
  parentContext: string;
  path: string;
  skillMdPath: string;
  contentHash: string;
  identicalCopyCount: number;
  allPaths: string[];
  variantLabel: string | null;
}

export interface ScanComplete {
  skills: SkillListItem[];
  dirsVisited: number;
  elapsedMs: number;
  errors: string[];
  cancelled: boolean;
}

export type ConflictPolicy = "skip" | "overwrite" | "rename";

export type CopyStatus = "success" | "skipped" | "error";

export interface CopyResult {
  source: string;
  destination: string;
  status: CopyStatus;
  message: string | null;
}

export interface AppSettings {
  scanRoot: string | null;
  recentDestinations: string[];
  ignorePatterns: string[];
}

export type ContextFilter = "all" | "claude" | "agents" | "cursor" | "other";

export interface FolderFileInfo {
  relativePath: string;
  absolutePath: string;
  sizeBytes: number;
  isBinary: boolean;
}

export interface FileContentResult {
  content: string;
  isBinary: boolean;
  truncated: boolean;
}
