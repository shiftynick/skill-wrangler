import { listSkillFiles, readSkillFile } from "../api";
import type { FolderFileInfo, SkillListItem } from "../types";
import { clearDiffState, initCompareForSkill } from "./diff.svelte";

export const previewState = $state({
  previewSkill: null as SkillListItem | null,
  previewFiles: [] as FolderFileInfo[],
  previewFilesLoading: false,
  selectedPreviewFile: null as FolderFileInfo | null,
  previewContent: "",
  previewContentLoading: false,
  previewTruncated: false,
});

export function clearPreview(): void {
  previewState.previewSkill = null;
  previewState.previewFiles = [];
  previewState.selectedPreviewFile = null;
  previewState.previewContent = "";
  previewState.previewTruncated = false;
  clearDiffState();
}

export async function selectSkillForPreview(skill: SkillListItem): Promise<void> {
  previewState.previewSkill = skill;
  initCompareForSkill(skill);
  previewState.previewFilesLoading = true;
  previewState.previewFiles = [];
  previewState.selectedPreviewFile = null;
  previewState.previewContent = "";
  previewState.previewTruncated = false;

  try {
    const files = await listSkillFiles(skill.path);
    previewState.previewFiles = files;
    const skillMd = files.find((f) => f.relativePath.toLowerCase() === "skill.md");
    if (skillMd) {
      await selectPreviewFile(skillMd);
    } else if (files.length > 0) {
      await selectPreviewFile(files[0]);
    }
  } catch (e) {
    previewState.previewContent = `Failed to list files: ${e}`;
  } finally {
    previewState.previewFilesLoading = false;
  }
}

export async function selectPreviewFile(file: FolderFileInfo): Promise<void> {
  previewState.selectedPreviewFile = file;
  previewState.previewContentLoading = true;
  previewState.previewContent = "";
  previewState.previewTruncated = false;
  try {
    const result = await readSkillFile(file.absolutePath);
    previewState.previewContent = result.content;
    previewState.previewTruncated = result.truncated;
  } catch (e) {
    previewState.previewContent = `Failed to read file: ${e}`;
  } finally {
    previewState.previewContentLoading = false;
  }
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
