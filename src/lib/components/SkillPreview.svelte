<script lang="ts">
  import {
    appState,
    formatBytes,
    getDisplayName,
    selectPreviewFile,
  } from "../stores/app.svelte";
</script>

<section class="panel-section preview-section">
  <h2>Preview</h2>
  {#if appState.previewSkill}
    <div class="meta">
      <strong>{getDisplayName(appState.previewSkill)}</strong>
      {#if appState.previewSkill.variantLabel}
        <span class="variant">{appState.previewSkill.variantLabel}</span>
      {/if}
      <span class="path">{appState.previewSkill.path}</span>
    </div>
    {#if appState.previewSkill.allPaths.length > 1}
      <div class="paths-block">
        <span class="paths-label">All copies:</span>
        <ul class="paths">
          {#each appState.previewSkill.allPaths as p}
            <li>{p}</li>
          {/each}
        </ul>
      </div>
    {/if}

    <div class="preview-layout">
      <div class="file-list">
        <h3>Files</h3>
        {#if appState.previewFilesLoading}
          <p class="hint">Loading files…</p>
        {:else if appState.previewFiles.length === 0}
          <p class="hint">No files found</p>
        {:else}
          <ul>
            {#each appState.previewFiles as file (file.absolutePath)}
              <li>
                <button
                  type="button"
                  class:active={appState.selectedPreviewFile?.absolutePath ===
                    file.absolutePath}
                  onclick={() => selectPreviewFile(file)}
                >
                  <span class="file-name">{file.relativePath}</span>
                  <span class="file-meta">
                    {formatBytes(file.sizeBytes)}{file.isBinary ? " · binary" : ""}
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      <div class="file-content">
        {#if appState.selectedPreviewFile}
          <div class="content-header">
            <span>{appState.selectedPreviewFile.relativePath}</span>
            {#if appState.previewTruncated}
              <span class="truncated">truncated</span>
            {/if}
          </div>
        {/if}
        {#if appState.previewContentLoading}
          <pre class="preview">Loading…</pre>
        {:else}
          <pre class="preview">{appState.previewContent || "Select a file to view"}</pre>
        {/if}
      </div>
    </div>
  {:else}
    <p class="empty">Select a skill to browse its folder</p>
  {/if}
</section>

<style>
  .preview-section {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
  }

  .meta {
    margin: 0 0 0.35rem;
    font-size: 0.8rem;
  }

  .paths-block {
    margin-bottom: 0.5rem;
  }

  .variant {
    display: block;
    color: var(--warn);
    margin-top: 0.15rem;
  }

  .path {
    display: block;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-muted);
    word-break: break-all;
    margin-top: 0.25rem;
  }

  .paths-label {
    display: block;
    font-size: 0.7rem;
    color: var(--text-muted);
    margin-top: 0.35rem;
  }

  .paths {
    margin: 0.15rem 0 0;
    padding-left: 1rem;
    font-family: var(--font-mono);
    font-size: 0.65rem;
    color: var(--text-muted);
    max-height: 4rem;
    overflow: auto;
  }

  .preview-layout {
    display: grid;
    grid-template-rows: auto 1fr;
    gap: 0.5rem;
    flex: 1;
    min-height: 0;
  }

  .file-list {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface-elevated);
    max-height: 160px;
    overflow: auto;
  }

  .file-list h3 {
    margin: 0;
    padding: 0.4rem 0.6rem;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--surface-elevated);
  }

  .file-list ul {
    list-style: none;
    margin: 0;
    padding: 0.25rem 0;
  }

  .file-list li button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 0.35rem 0.6rem;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .file-list li button:hover,
  .file-list li button.active {
    background: var(--row-selected);
  }

  .file-name {
    font-family: var(--font-mono);
    word-break: break-all;
  }

  .file-meta {
    font-size: 0.65rem;
    color: var(--text-muted);
  }

  .file-content {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
  }

  .content-header {
    display: flex;
    justify-content: space-between;
    font-size: 0.7rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
    margin-bottom: 0.35rem;
  }

  .truncated {
    color: var(--warn);
  }

  .preview {
    flex: 1;
    margin: 0;
    padding: 0.75rem;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    line-height: 1.45;
    background: var(--surface-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    white-space: pre-wrap;
    word-break: break-word;
    min-height: 120px;
  }

  .hint,
  .empty {
    color: var(--text-muted);
    font-size: 0.85rem;
    padding: 0.5rem 0.6rem;
  }
</style>
