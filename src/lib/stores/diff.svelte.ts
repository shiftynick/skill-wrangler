import { diffSkillFile, diffSkillFolders } from "../api";
import type { FileDiffEntry, FileDiffStatus, SkillFolderDiff, TextFileDiff } from "../types";
import { getSkillById, getVariantsFor } from "./scan.svelte";
import { setError } from "./ui.svelte";

export const diffState = $state({
  modalOpen: false,
  leftSkillId: null as string | null,
  rightSkillId: null as string | null,
  folderDiff: null as SkillFolderDiff | null,
  folderDiffLoading: false,
  selectedDiffPath: null as string | null,
  fileDiff: null as TextFileDiff | null,
  fileDiffLoading: false,
  showUnchanged: false,
});

export function clearDiffResults(): void {
  diffState.folderDiff = null;
  diffState.selectedDiffPath = null;
  diffState.fileDiff = null;
}

export function closeCompareModal(): void {
  diffState.modalOpen = false;
}

export function openCompareModal(): void {
  if (!diffState.leftSkillId || !diffState.rightSkillId) return;
  diffState.modalOpen = true;
}

export function clearDiffState(): void {
  diffState.modalOpen = false;
  diffState.leftSkillId = null;
  diffState.rightSkillId = null;
  diffState.folderDiffLoading = false;
  diffState.fileDiffLoading = false;
  diffState.showUnchanged = false;
  clearDiffResults();
}

export function initCompareForSkill(skill: import("../types").SkillListItem): void {
  const variants = getVariantsFor(skill);
  diffState.leftSkillId = skill.id;
  const sibling = variants.find((v) => v.id !== skill.id);
  diffState.rightSkillId = sibling?.id ?? skill.id;
  clearDiffResults();
}

export function getCompareVariants(): import("../types").SkillListItem[] {
  const skill = diffState.leftSkillId ? getSkillById(diffState.leftSkillId) : null;
  if (!skill) return [];
  return getVariantsFor(skill);
}

export function statusLabel(status: FileDiffStatus): string {
  switch (status) {
    case "added":
      return "+";
    case "removed":
      return "−";
    case "modified":
      return "~";
    case "binary_changed":
      return "bin";
    default:
      return "=";
  }
}

export function isDiffableStatus(status: FileDiffStatus): boolean {
  return status === "added" || status === "removed" || status === "modified";
}

export function getVisibleDiffFiles(): FileDiffEntry[] {
  const files = diffState.folderDiff?.files ?? [];
  if (diffState.showUnchanged) return files;
  return files.filter((f) => f.status !== "unchanged");
}

export async function runFolderDiff(): Promise<void> {
  const left = diffState.leftSkillId ? getSkillById(diffState.leftSkillId) : null;
  const right = diffState.rightSkillId ? getSkillById(diffState.rightSkillId) : null;
  if (!left || !right) {
    setError("Select left and right variants to compare");
    return;
  }
  if (left.id === right.id) {
    setError("Choose two different variants to compare");
    return;
  }

  setError(null);
  diffState.folderDiffLoading = true;
  clearDiffResults();
  try {
    diffState.folderDiff = await diffSkillFolders(left.path, right.path);
    const firstChanged = getVisibleDiffFiles().find((f) => isDiffableStatus(f.status));
    if (firstChanged) {
      await selectDiffFile(firstChanged);
    }
  } catch (e) {
    setError(String(e));
  } finally {
    diffState.folderDiffLoading = false;
  }
}

export async function selectDiffFile(entry: FileDiffEntry): Promise<void> {
  diffState.selectedDiffPath = entry.relativePath;
  diffState.fileDiff = null;

  if (!isDiffableStatus(entry.status)) {
    return;
  }

  const left = diffState.leftSkillId ? getSkillById(diffState.leftSkillId) : null;
  const right = diffState.rightSkillId ? getSkillById(diffState.rightSkillId) : null;
  if (!left || !right) return;

  diffState.fileDiffLoading = true;
  try {
    diffState.fileDiff = await diffSkillFile(left.path, right.path, entry.relativePath);
  } catch (e) {
    diffState.fileDiff = {
      relativePath: entry.relativePath,
      truncated: false,
      lines: [{ tag: "equal", content: String(e) }],
    };
  } finally {
    diffState.fileDiffLoading = false;
  }
}
