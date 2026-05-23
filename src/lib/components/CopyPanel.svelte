<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    appState,
    copySelected,
    setCopyToAllSkillFolders,
    setDestination,
  } from "../stores/app.svelte";
  import type { ConflictPolicy } from "../types";

  const conflictOptions: { value: ConflictPolicy; label: string }[] = [
    { value: "rename", label: "Rename if exists" },
    { value: "skip", label: "Skip if exists" },
    { value: "overwrite", label: "Overwrite" },
  ];

  async function pickDestination() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose destination folder",
    });
    if (selected && typeof selected === "string") {
      await setDestination(selected);
    }
  }

  function pickRecent(path: string) {
    void setDestination(path);
  }

  function statusClass(status: string): string {
    return `log-${status}`;
  }

  function relativeDest(fullPath: string): string {
    const root = appState.destination.replace(/\\/g, "/").replace(/\/$/, "");
    const norm = fullPath.replace(/\\/g, "/");
    if (norm.toLowerCase().startsWith(root.toLowerCase())) {
      return norm.slice(root.length).replace(/^\//, "") || fullPath;
    }
    return fullPath;
  }
</script>

<section class="panel-section">
  <h2>Copy to</h2>
  <p class="hint">Copies entire skill folders into the destination</p>

  <div class="path-row">
    <input
      type="text"
      readonly
      value={appState.destination}
      placeholder="No destination selected"
    />
    <button type="button" onclick={pickDestination}>Browse</button>
  </div>

  {#if appState.recentDestinations.length > 0}
    <div class="recent">
      <span class="label">Recent:</span>
      {#each appState.recentDestinations.slice(0, 5) as path}
        <button type="button" class="chip" onclick={() => pickRecent(path)} title={path}>
          {path.split(/[/\\]/).pop() ?? path}
        </button>
      {/each}
    </div>
  {/if}

  <label class="checkbox-field">
    <input
      type="checkbox"
      checked={appState.copyToAllSkillFolders}
      onchange={(e) => setCopyToAllSkillFolders(e.currentTarget.checked)}
    />
    <span>Copy to all skill folders under destination</span>
  </label>
  {#if appState.copyToAllSkillFolders && appState.destination}
    <p class="subhint">
      {#if appState.discoveredSkillContainers.length === 0}
        No skill folders found yet — will copy into the destination root.
      {:else}
        Found {appState.discoveredSkillContainers.length} skill folder{appState.discoveredSkillContainers.length === 1 ? "" : "s"}:
      {/if}
    </p>
    {#if appState.discoveredSkillContainers.length > 0}
      <ul class="container-list">
        {#each appState.discoveredSkillContainers as container}
          <li title={container}>{relativeDest(container)}</li>
        {/each}
      </ul>
    {/if}
  {/if}

  <label class="field">
    <span>On conflict</span>
    <select bind:value={appState.conflictPolicy}>
      {#each conflictOptions as opt}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </label>

  <button
    type="button"
    class="primary copy-btn"
    disabled={appState.copying || appState.selectedIds.size === 0}
    onclick={copySelected}
  >
    {appState.copying
      ? "Copying…"
      : `Copy ${appState.selectedIds.size} skill(s)`}
  </button>

  {#if appState.copyLog.length > 0}
    <div class="log">
      <h3>Results</h3>
      <ul>
        {#each appState.copyLog as result}
          <li class={statusClass(result.status)}>
            <span class="status">{result.status}</span>
            <span class="dest" title={result.destination}>
              {relativeDest(result.destination)}
            </span>
            {#if result.message}
              <span class="msg">{result.message}</span>
            {/if}
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</section>

<style>
  .hint {
    margin: 0 0 0.75rem;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .subhint {
    margin: 0 0 0.35rem;
    font-size: 0.75rem;
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

  .recent {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    margin-top: 0.5rem;
  }

  .recent .label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .chip {
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    border-radius: 999px;
    background: var(--surface-elevated);
    border: 1px solid var(--border);
    cursor: pointer;
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip:hover {
    border-color: var(--accent);
  }

  .checkbox-field {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    margin: 0.75rem 0 0.35rem;
    font-size: 0.8rem;
    cursor: pointer;
  }

  .checkbox-field input {
    margin-top: 0.15rem;
    flex-shrink: 0;
  }

  .container-list {
    margin: 0 0 0.5rem;
    padding-left: 1.25rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-muted);
    max-height: 80px;
    overflow: auto;
  }

  .container-list li {
    margin: 0.15rem 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin: 0.75rem 0;
    font-size: 0.85rem;
  }

  .field select {
    width: 100%;
  }

  .copy-btn {
    width: 100%;
    margin-top: 0.25rem;
  }

  .log {
    margin-top: 1rem;
    border-top: 1px solid var(--border);
    padding-top: 0.75rem;
  }

  .log h3 {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
  }

  .log ul {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 200px;
    overflow: auto;
  }

  .log li {
    font-size: 0.75rem;
    padding: 0.35rem 0;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    align-items: baseline;
  }

  .status {
    text-transform: uppercase;
    font-weight: 600;
    font-size: 0.65rem;
  }

  .log-success .status {
    color: var(--success);
  }

  .log-skipped .status {
    color: var(--warn);
  }

  .log-error .status {
    color: var(--error);
  }

  .dest {
    font-family: var(--font-mono);
    word-break: break-all;
  }

  .msg {
    color: var(--text-muted);
    width: 100%;
  }
</style>
