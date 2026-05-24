import { addRecentDestination, copySkills, findSkillContainers, getSettings } from "../api";
import type { ConflictPolicy, CopyResult } from "../types";
import { scanState } from "./scan.svelte";
import { setError } from "./ui.svelte";

export const copyState = $state({
  destination: "",
  conflictPolicy: "rename" as ConflictPolicy,
  recentDestinations: [] as string[],
  copyLog: [] as CopyResult[],
  copying: false,
  copyToAllSkillFolders: false,
  discoveredSkillContainers: [] as string[],
});

export async function loadCopySettings(): Promise<void> {
  const settings = await getSettings();
  copyState.recentDestinations = settings.recentDestinations;
}

export async function refreshSkillContainers(): Promise<void> {
  if (!copyState.destination || !copyState.copyToAllSkillFolders) {
    copyState.discoveredSkillContainers = [];
    return;
  }
  try {
    copyState.discoveredSkillContainers = await findSkillContainers(
      copyState.destination,
    );
  } catch {
    copyState.discoveredSkillContainers = [];
  }
}

export async function setDestination(path: string): Promise<void> {
  copyState.destination = path;
  try {
    await addRecentDestination(path);
    const settings = await getSettings();
    copyState.recentDestinations = settings.recentDestinations;
  } catch {
    /* non-fatal */
  }
  await refreshSkillContainers();
}

export function setCopyToAllSkillFolders(enabled: boolean): void {
  copyState.copyToAllSkillFolders = enabled;
  void refreshSkillContainers();
}

export async function copySelected(): Promise<void> {
  if (!copyState.destination) {
    setError("Choose a destination folder first");
    return;
  }
  const selected = scanState.skills.filter((s) => scanState.selectedIds.has(s.id));
  if (selected.length === 0) {
    setError("Select at least one skill to copy");
    return;
  }

  copyState.copying = true;
  setError(null);
  copyState.copyLog = [];
  try {
    const results = await copySkills(
      selected.map((s) => s.path),
      copyState.destination,
      copyState.conflictPolicy,
      copyState.copyToAllSkillFolders,
    );
    copyState.copyLog = results;
  } catch (e) {
    setError(String(e));
  } finally {
    copyState.copying = false;
  }
}
