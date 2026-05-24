<script lang="ts">
  import {
    closeCompareModal,
    diffState,
    getCompareVariants,
    getVisibleDiffFiles,
    isDiffableStatus,
    previewState,
    runFolderDiff,
    selectDiffFile,
    statusLabel,
  } from "../stores/app.svelte";

  const compareVariants = $derived(getCompareVariants());
  const visibleDiffFiles = $derived(getVisibleDiffFiles());
  const changedCount = $derived(
    diffState.folderDiff?.files.filter((f) => f.status !== "unchanged").length ?? 0,
  );
  const folderName = $derived(previewState.previewSkill?.folderName ?? "skill");

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeCompareModal();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && diffState.modalOpen) {
      closeCompareModal();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if diffState.modalOpen}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="compare-modal-title"
    >
      <header class="modal-header">
        <div class="modal-title-block">
          <h2 id="compare-modal-title">Compare variants — {folderName}</h2>
          <p class="modal-subtitle">
            {compareVariants.length} content variant{compareVariants.length === 1 ? "" : "s"}
          </p>
        </div>
        <button type="button" class="close-btn" onclick={() => closeCompareModal()} aria-label="Close">
          ×
        </button>
      </header>

      <div class="compare-controls">
        <label>
          Left
          <select bind:value={diffState.leftSkillId}>
            {#each compareVariants as v (v.id)}
              <option value={v.id}>{v.variantLabel ?? v.parentContext}</option>
            {/each}
          </select>
        </label>
        <label>
          Right
          <select bind:value={diffState.rightSkillId}>
            {#each compareVariants as v (v.id)}
              <option value={v.id}>{v.variantLabel ?? v.parentContext}</option>
            {/each}
          </select>
        </label>
        <button
          type="button"
          class="primary"
          disabled={diffState.folderDiffLoading ||
            diffState.leftSkillId === diffState.rightSkillId}
          onclick={() => runFolderDiff()}
        >
          {diffState.folderDiffLoading ? "Comparing…" : "Compare"}
        </button>
      </div>

      <div class="modal-body">
        {#if diffState.folderDiff}
          <div class="diff-summary">
            {changedCount} changed file{changedCount === 1 ? "" : "s"}
            <label class="show-unchanged">
              <input type="checkbox" bind:checked={diffState.showUnchanged} />
              Show unchanged
            </label>
          </div>

          <div class="diff-layout">
            <div class="diff-file-list">
              <h3>Changes</h3>
              {#if visibleDiffFiles.length === 0}
                <p class="hint">No differences</p>
              {:else}
                <ul>
                  {#each visibleDiffFiles as entry (entry.relativePath)}
                    <li>
                      <button
                        type="button"
                        class:active={diffState.selectedDiffPath === entry.relativePath}
                        onclick={() => selectDiffFile(entry)}
                      >
                        <span class="status status-{entry.status}">{statusLabel(entry.status)}</span>
                        <span class="file-name">{entry.relativePath}</span>
                      </button>
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>

            <div class="diff-content">
              {#if diffState.selectedDiffPath}
                <div class="content-header">
                  <span>{diffState.selectedDiffPath}</span>
                  {#if diffState.fileDiff?.truncated}
                    <span class="truncated">diff may be incomplete (file truncated)</span>
                  {/if}
                </div>
              {/if}
              {#if diffState.fileDiffLoading}
                <pre class="diff-preview">Loading diff…</pre>
              {:else if diffState.fileDiff}
                <div class="diff-preview diff-lines">
                  {#each diffState.fileDiff.lines as line, i (i)}
                    <div class="diff-line diff-{line.tag}">{line.content || "\u00a0"}</div>
                  {/each}
                </div>
              {:else if diffState.selectedDiffPath}
                {@const entry = diffState.folderDiff?.files.find(
                  (f) => f.relativePath === diffState.selectedDiffPath,
                )}
                {#if entry?.status === "binary_changed"}
                  <pre class="diff-preview hint">Binary file changed (no line diff)</pre>
                {:else if entry?.status === "unchanged"}
                  <pre class="diff-preview hint">File unchanged</pre>
                {:else}
                  <pre class="diff-preview hint">Select a changed file to view diff</pre>
                {/if}
              {:else}
                <pre class="diff-preview hint">Select a file from the list</pre>
              {/if}
            </div>
          </div>
        {:else}
          <p class="hint empty-state">
            Pick left and right variants, then click Compare to see file differences.
          </p>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(2px);
  }

  .modal {
    display: flex;
    flex-direction: column;
    width: min(1100px, 100%);
    height: min(85vh, 900px);
    max-height: 100%;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.45);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .modal-title-block h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .modal-subtitle {
    margin: 0.25rem 0 0;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .close-btn {
    flex-shrink: 0;
    width: 2rem;
    height: 2rem;
    padding: 0;
    font-size: 1.35rem;
    line-height: 1;
    border-radius: 6px;
  }

  .compare-controls {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: flex-end;
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
    flex-shrink: 0;
  }

  .compare-controls label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--text-muted);
  }

  .compare-controls select {
    min-width: 12rem;
    font-size: 0.8rem;
  }

  .compare-controls .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }

  .compare-controls .primary:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    padding: 0.75rem 1.25rem 1.25rem;
    gap: 0.5rem;
  }

  .diff-summary {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.8rem;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .show-unchanged {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    cursor: pointer;
  }

  .diff-layout {
    display: grid;
    grid-template-columns: minmax(180px, 240px) 1fr;
    gap: 0.75rem;
    flex: 1;
    min-height: 0;
  }

  .diff-file-list {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface-elevated);
    overflow: auto;
    min-height: 0;
  }

  .diff-file-list h3 {
    margin: 0;
    padding: 0.5rem 0.75rem;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--surface-elevated);
    z-index: 1;
  }

  .diff-file-list ul {
    list-style: none;
    margin: 0;
    padding: 0.25rem 0;
  }

  .diff-file-list li button {
    display: flex;
    align-items: flex-start;
    gap: 0.4rem;
    width: 100%;
    padding: 0.4rem 0.75rem;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .diff-file-list li button:hover,
  .diff-file-list li button.active {
    background: var(--row-selected);
  }

  .status {
    flex-shrink: 0;
    font-family: var(--font-mono);
    font-weight: 600;
    width: 1.5rem;
    text-align: center;
  }

  .status-added {
    color: var(--success);
  }

  .status-removed {
    color: var(--error);
  }

  .status-modified {
    color: var(--warn);
  }

  .status-binary_changed {
    color: var(--text-muted);
  }

  .file-name {
    font-family: var(--font-mono);
    word-break: break-all;
  }

  .diff-content {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .content-header {
    display: flex;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.35rem;
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
    margin-bottom: 0.35rem;
    flex-shrink: 0;
  }

  .truncated {
    color: var(--warn);
  }

  .diff-preview {
    flex: 1;
    margin: 0;
    padding: 0.75rem;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: 0.8rem;
    line-height: 1.45;
    background: var(--surface-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    min-height: 0;
  }

  .diff-lines {
    display: flex;
    flex-direction: column;
  }

  .diff-line {
    white-space: pre-wrap;
    word-break: break-word;
    min-height: 1.45em;
  }

  .diff-insert {
    background: color-mix(in srgb, var(--success) 18%, transparent);
  }

  .diff-delete {
    background: color-mix(in srgb, var(--error) 18%, transparent);
  }

  .hint {
    color: var(--text-muted);
    font-size: 0.85rem;
    padding: 0.5rem 0.75rem;
  }

  .empty-state {
    margin: auto;
    text-align: center;
    max-width: 24rem;
  }
</style>
