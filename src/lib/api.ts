import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  ConflictPolicy,
  CopyResult,
  FileContentResult,
  FolderFileInfo,
} from "./types";

export async function getScanRoot(): Promise<string | null> {
  return invoke("get_scan_root");
}

export async function setScanRoot(path: string): Promise<void> {
  return invoke("set_scan_root", { path });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function addRecentDestination(path: string): Promise<void> {
  return invoke("add_recent_destination", { path });
}

export async function startScan(): Promise<void> {
  return invoke("start_scan");
}

export async function cancelScan(): Promise<void> {
  return invoke("cancel_scan");
}

export async function copySkills(
  sources: string[],
  dest: string,
  onConflict: ConflictPolicy,
  copyToAllSkillFolders = false,
): Promise<CopyResult[]> {
  return invoke("copy_skills_command", {
    sources,
    dest,
    onConflict,
    copyToAllSkillFolders,
  });
}

export async function findSkillContainers(root: string): Promise<string[]> {
  return invoke("find_skill_containers_command", { root });
}

export async function listSkillFiles(skillPath: string): Promise<FolderFileInfo[]> {
  return invoke("list_skill_files_command", { skillPath });
}

export async function readSkillFile(
  path: string,
  maxBytes = 512 * 1024,
): Promise<FileContentResult> {
  return invoke("read_skill_file_command", { path, maxBytes });
}

export async function initAgentSkillsFolders(destination: string): Promise<string[]> {
  return invoke("init_agent_skills_folders", { destination });
}
