export { copyState, copySelected, refreshSkillContainers, setCopyToAllSkillFolders, setDestination, loadCopySettings } from "./copy.svelte";
export { previewState, clearPreview, formatBytes, selectPreviewFile, selectSkillForPreview } from "./preview.svelte";
export {
  scanState,
  chooseScanRoot,
  clearSelection,
  getContextFilterOptions,
  getFilteredSkills,
  initScanListeners,
  loadScanSettings,
  runScan,
  selectAllFiltered,
  stopScan,
  toggleSelection,
} from "./scan.svelte";
export { setError, uiState } from "./ui.svelte";

import { loadCopySettings } from "./copy.svelte";
import { initScanListeners, loadScanSettings } from "./scan.svelte";
import { setError } from "./ui.svelte";

export async function initApp(): Promise<void> {
  await initScanListeners();
  try {
    await Promise.all([loadScanSettings(), loadCopySettings()]);
  } catch (e) {
    setError(String(e));
  }
}
