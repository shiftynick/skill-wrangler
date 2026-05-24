<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { chooseScanRoot, runScan, scanState, stopScan } from "../stores/scan.svelte";

  async function pickRoot() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose scan root folder",
    });
    if (selected && typeof selected === "string") {
      await chooseScanRoot(selected);
    }
  }
</script>

<section class="panel-section">
  <h2>Scan root</h2>
  <p class="hint">Recursively finds folders containing SKILL.md</p>

  <div class="path-row">
    <input
      type="text"
      readonly
      value={scanState.scanRoot ?? ""}
      placeholder="No folder selected"
    />
    <button type="button" onclick={pickRoot}>Browse</button>
  </div>

  <div class="actions">
    <button
      type="button"
      class="primary"
      disabled={!scanState.scanRoot || scanState.scanning}
      onclick={runScan}
    >
      {scanState.scanning ? "Scanning…" : "Rescan"}
    </button>
    {#if scanState.scanning}
      <button type="button" class="ghost" onclick={stopScan}>Cancel</button>
    {/if}
  </div>

  {#if scanState.scanStats}
    <div class="stats">
      <span>{scanState.skills.length} skills</span>
      <span>{scanState.scanStats.dirsVisited} dirs</span>
      <span>{(scanState.scanStats.elapsedMs / 1000).toFixed(1)}s</span>
      {#if scanState.scanStats.cancelled}
        <span class="warn">cancelled</span>
      {/if}
    </div>
    {#if scanState.scanStats.errors.length > 0}
      <p class="warn small">
        {scanState.scanStats.errors.length} warning(s) during scan
      </p>
    {/if}
  {/if}
</section>

<style>
  .hint {
    margin: 0 0 0.75rem;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .path-row {
    display: flex;
    gap: 0.5rem;
  }

  .path-row input {
    flex: 1;
    min-width: 0;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-top: 0.75rem;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .warn {
    color: var(--warn);
  }

  .small {
    font-size: 0.75rem;
    margin: 0.5rem 0 0;
  }
</style>
