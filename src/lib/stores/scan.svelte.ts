import { listen } from "@tauri-apps/api/event";
import { cancelScan, getSettings, setScanRoot, startScan } from "../api";
import type { ContextFilter, ScanComplete, SkillListItem } from "../types";
import { clearPreview } from "./preview.svelte";
import { setError } from "./ui.svelte";

export const scanState = $state({
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
  agentSkillRoots: [] as string[],
});

let listenersReady = false;

export function agentFilterKey(root: string): string {
  return root.startsWith(".") ? root.slice(1) : root;
}

export function matchesContext(skill: SkillListItem, filter: ContextFilter): boolean {
  if (filter === "all") return true;
  const ctx = skill.parentContext.toLowerCase();
  if (filter === "other") {
    return !scanState.agentSkillRoots.some((root) =>
      ctx.includes(root.toLowerCase()),
    );
  }
  const root = scanState.agentSkillRoots.find((r) => agentFilterKey(r) === filter);
  return root ? ctx.includes(root.toLowerCase()) : false;
}

export function getContextFilterOptions(): { value: ContextFilter; label: string }[] {
  const options: { value: ContextFilter; label: string }[] = [
    { value: "all", label: "All" },
    ...scanState.agentSkillRoots.map((root) => ({
      value: agentFilterKey(root) as ContextFilter,
      label: root,
    })),
    { value: "other", label: "Other" },
  ];
  return options;
}

export function getFilteredSkills(): SkillListItem[] {
  return scanState.skills.filter((skill) => {
    if (!matchesContext(skill, scanState.contextFilter)) return false;
    const q = scanState.searchQuery.trim().toLowerCase();
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

export async function loadScanSettings(): Promise<void> {
  const settings = await getSettings();
  scanState.scanRoot = settings.scanRoot;
  scanState.agentSkillRoots = settings.agentSkillRoots;
}

export async function initScanListeners(): Promise<void> {
  if (!listenersReady) {
    await listen<ScanComplete>("scan-complete", (event) => {
      scanState.scanning = false;
      scanState.skills = event.payload.skills;
      scanState.selectedIds = new Set();
      scanState.scanStats = {
        dirsVisited: event.payload.dirsVisited,
        elapsedMs: event.payload.elapsedMs,
        errors: event.payload.errors,
        cancelled: event.payload.cancelled,
      };
      if (event.payload.errors.length > 0 && event.payload.skills.length === 0) {
        setError(event.payload.errors[0] ?? "Scan failed");
      }
    });
    listenersReady = true;
  }
}

export async function chooseScanRoot(path: string): Promise<void> {
  setError(null);
  await setScanRoot(path);
  scanState.scanRoot = path;
  await runScan();
}

export async function runScan(): Promise<void> {
  if (!scanState.scanRoot) {
    setError("Choose a scan root folder first");
    return;
  }
  setError(null);
  scanState.scanning = true;
  scanState.scanStats = null;
  scanState.skills = [];
  scanState.selectedIds = new Set();
  clearPreview();
  try {
    await startScan();
  } catch (e) {
    scanState.scanning = false;
    setError(String(e));
  }
}

export async function stopScan(): Promise<void> {
  try {
    await cancelScan();
  } catch (e) {
    setError(String(e));
  }
}

export function toggleSelection(id: string): void {
  const next = new Set(scanState.selectedIds);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  scanState.selectedIds = next;
}

export function selectAllFiltered(): void {
  scanState.selectedIds = new Set(getFilteredSkills().map((s) => s.id));
}

export function clearSelection(): void {
  scanState.selectedIds = new Set();
}
