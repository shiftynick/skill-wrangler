import { listen } from "@tauri-apps/api/event";
import {
  addRecentDestination,
  cancelScan,
  copySkills,
  findSkillContainers,
  getSettings,
  listSkillFiles,
  readSkillFile,
  setScanRoot,
  startScan,
} from "../api";
import type {
  AppSettings,
  ConflictPolicy,
  ContextFilter,
  CopyResult,
  FolderFileInfo,
  ScanComplete,
  SkillListItem,
} from "../types";

export const appState = $state({
  scanRoot: null as string | null,
  skills: [] as SkillListItem[],
  selectedIds: new Set<string>(),
  searchQuery: "",
  contextFilter: "all" as ContextFilter,
  scanning: false,
  scanStats: null as {
    dirsVisited: number;
    elapsedMs: number;
    errors: string[];
    cancelled: boolean;
  } | null,
  previewSkill: null as SkillListItem | null,
  previewFiles: [] as FolderFileInfo[],
  previewFilesLoading: false,
  selectedPreviewFile: null as FolderFileInfo | null,
  previewContent: "",
  previewContentLoading: false,
  previewTruncated: false,
  destination: "",
  conflictPolicy: "rename" as ConflictPolicy,
  recentDestinations: [] as string[],
  copyLog: [] as CopyResult[],
  copying: false,
  copyToAllSkillFolders: false,
  discoveredSkillContainers: [] as string[],
  compactList: false,
  error: null as string | null,
});

let listenersReady = false;

export function getDisplayName(skill: SkillListItem): string {
  return skill.folderName;
}

export function matchesContext(skill: SkillListItem, filter: ContextFilter): boolean {
  if (filter === "all") return true;
  const ctx = skill.parentContext.toLowerCase();
  if (filter === "claude") return ctx.includes(".claude");
  if (filter === "agents") return ctx.includes(".agents");
  if (filter === "cursor") return ctx.includes(".cursor");
  return (
    !ctx.includes(".claude") &&
    !ctx.includes(".agents") &&
    !ctx.includes(".cursor")
  );
}

export function getFilteredSkills(): SkillListItem[] {
  return appState.skills.filter((skill) => {
    if (!matchesContext(skill, appState.contextFilter)) return false;
    const q = appState.searchQuery.trim().toLowerCase();
    if (!q) return true;
    const haystack = [
      skill.folderName,
      skill.description ?? "",
      skill.path,
      skill.parentContext,
      skill.variantLabel ?? "",
      ...skill.allPaths,
    ]
      .join(" ")
      .toLowerCase();
    return haystack.includes(q);
  });
}

export async function initApp(): Promise<void> {
  if (!listenersReady) {
    await listen<ScanComplete>("scan-complete", (event) => {
      appState.scanning = false;
      appState.skills = event.payload.skills;
      appState.selectedIds = new Set();
      appState.scanStats = {
        dirsVisited: event.payload.dirsVisited,
        elapsedMs: event.payload.elapsedMs,
        errors: event.payload.errors,
        cancelled: event.payload.cancelled,
      };
      if (event.payload.errors.length > 0 && event.payload.skills.length === 0) {
        appState.error = event.payload.errors[0] ?? "Scan failed";
      }
    });
    listenersReady = true;
  }

  try {
    const settings: AppSettings = await getSettings();
    appState.scanRoot = settings.scanRoot;
    appState.recentDestinations = settings.recentDestinations;
  } catch (e) {
    appState.error = String(e);
  }
}

export async function chooseScanRoot(path: string): Promise<void> {
  appState.error = null;
  await setScanRoot(path);
  appState.scanRoot = path;
  await runScan();
}

export async function runScan(): Promise<void> {
  if (!appState.scanRoot) {
    appState.error = "Choose a scan root folder first";
    return;
  }
  appState.error = null;
  appState.scanning = true;
  appState.scanStats = null;
  appState.skills = [];
  appState.selectedIds = new Set();
  appState.previewSkill = null;
  appState.previewFiles = [];
  appState.selectedPreviewFile = null;
  appState.previewContent = "";
  try {
    await startScan();
  } catch (e) {
    appState.scanning = false;
    appState.error = String(e);
  }
}

export async function stopScan(): Promise<void> {
  try {
    await cancelScan();
  } catch (e) {
    appState.error = String(e);
  }
}

export function toggleSelection(id: string): void {
  const next = new Set(appState.selectedIds);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  appState.selectedIds = next;
}

export function selectAllFiltered(): void {
  appState.selectedIds = new Set(getFilteredSkills().map((s) => s.id));
}

export function clearSelection(): void {
  appState.selectedIds = new Set();
}

export async function selectSkillForPreview(skill: SkillListItem): Promise<void> {
  appState.previewSkill = skill;
  appState.previewFilesLoading = true;
  appState.previewFiles = [];
  appState.selectedPreviewFile = null;
  appState.previewContent = "";
  appState.previewTruncated = false;

  try {
    const files = await listSkillFiles(skill.path);
    appState.previewFiles = files;
    const skillMd = files.find((f) => f.relativePath.toLowerCase() === "skill.md");
    if (skillMd) {
      await selectPreviewFile(skillMd);
    } else if (files.length > 0) {
      await selectPreviewFile(files[0]);
    }
  } catch (e) {
    appState.previewContent = `Failed to list files: ${e}`;
  } finally {
    appState.previewFilesLoading = false;
  }
}

export async function selectPreviewFile(file: FolderFileInfo): Promise<void> {
  appState.selectedPreviewFile = file;
  appState.previewContentLoading = true;
  appState.previewContent = "";
  appState.previewTruncated = false;
  try {
    const result = await readSkillFile(file.absolutePath);
    appState.previewContent = result.content;
    appState.previewTruncated = result.truncated;
  } catch (e) {
    appState.previewContent = `Failed to read file: ${e}`;
  } finally {
    appState.previewContentLoading = false;
  }
}

export async function refreshSkillContainers(): Promise<void> {
  if (!appState.destination || !appState.copyToAllSkillFolders) {
    appState.discoveredSkillContainers = [];
    return;
  }
  try {
    appState.discoveredSkillContainers = await findSkillContainers(
      appState.destination,
    );
  } catch {
    appState.discoveredSkillContainers = [];
  }
}

export async function setDestination(path: string): Promise<void> {
  appState.destination = path;
  try {
    await addRecentDestination(path);
    const settings = await getSettings();
    appState.recentDestinations = settings.recentDestinations;
  } catch {
    /* non-fatal */
  }
  await refreshSkillContainers();
}

export function setCopyToAllSkillFolders(enabled: boolean): void {
  appState.copyToAllSkillFolders = enabled;
  void refreshSkillContainers();
}

export async function copySelected(): Promise<void> {
  if (!appState.destination) {
    appState.error = "Choose a destination folder first";
    return;
  }
  const selected = appState.skills.filter((s) => appState.selectedIds.has(s.id));
  if (selected.length === 0) {
    appState.error = "Select at least one skill to copy";
    return;
  }

  appState.copying = true;
  appState.error = null;
  appState.copyLog = [];
  try {
    const results = await copySkills(
      selected.map((s) => s.path),
      appState.destination,
      appState.conflictPolicy,
      appState.copyToAllSkillFolders,
    );
    appState.copyLog = results;
  } catch (e) {
    appState.error = String(e);
  } finally {
    appState.copying = false;
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export { formatBytes };
