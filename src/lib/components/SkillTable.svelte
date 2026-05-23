<script lang="ts">
  import {
    appState,
    clearSelection,
    getDisplayName,
    getFilteredSkills,
    selectAllFiltered,
    selectSkillForPreview,
    toggleSelection,
  } from "../stores/app.svelte";
  import type { ContextFilter } from "../types";

  const filteredSkills = $derived(getFilteredSkills());

  const contextOptions: { value: ContextFilter; label: string }[] = [
    { value: "all", label: "All" },
    { value: "claude", label: ".claude" },
    { value: "agents", label: ".agents" },
    { value: "cursor", label: ".cursor" },
    { value: "other", label: "Other" },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "a") {
      e.preventDefault();
      selectAllFiltered();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<section class="panel-section list-section">
  <div class="toolbar">
    <input
      type="search"
      placeholder="Search skills…"
      bind:value={appState.searchQuery}
    />
    <select bind:value={appState.contextFilter}>
      {#each contextOptions as opt}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
    <label class="compact-toggle" title="Show skill names only">
      <input type="checkbox" bind:checked={appState.compactList} />
      Compact
    </label>
    <button type="button" class="ghost" onclick={selectAllFiltered}>Select all</button>
    <button type="button" class="ghost" onclick={clearSelection}>Clear</button>
    <span class="count">{appState.selectedIds.size} selected · {filteredSkills.length} skills</span>
  </div>

  {#if appState.scanning}
    <div class="empty">Scanning filesystem…</div>
  {:else if !appState.scanRoot}
    <div class="empty">Choose a scan root to begin</div>
  {:else if filteredSkills.length === 0}
    <div class="empty">No skills match your filters</div>
  {:else}
    <div class="list-wrap" class:compact={appState.compactList}>
      <ul class="skill-list">
        {#each filteredSkills as skill (skill.id)}
          <li
            class:selected={appState.selectedIds.has(skill.id)}
            class:active={appState.previewSkill?.id === skill.id}
            class:compact={appState.compactList}
            title={appState.compactList
              ? [skill.variantLabel, skill.description, skill.parentContext]
                  .filter(Boolean)
                  .join(" · ")
              : undefined}
          >
            <input
              type="checkbox"
              checked={appState.selectedIds.has(skill.id)}
              onclick={(e) => e.stopPropagation()}
              onchange={() => toggleSelection(skill.id)}
            />
            <button
              type="button"
              class="skill-btn"
              class:compact={appState.compactList}
              onclick={() => selectSkillForPreview(skill)}
            >
              <span class="name">{getDisplayName(skill)}</span>
              {#if !appState.compactList}
                {#if skill.variantLabel}
                  <span class="variant">{skill.variantLabel}</span>
                {/if}
                {#if skill.description}
                  <span class="desc">{skill.description}</span>
                {/if}
                <span class="context">{skill.parentContext}</span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</section>

<style>
  .list-section {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .toolbar input[type="search"] {
    flex: 1;
    min-width: 140px;
  }

  .count {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin-left: auto;
  }

  .compact-toggle {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.8rem;
    color: var(--text-muted);
    cursor: pointer;
    user-select: none;
  }

  .compact-toggle input {
    margin: 0;
  }

  .list-wrap {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
  }

  .skill-list {
    list-style: none;
    margin: 0;
    padding: 0.25rem 0;
  }

  .skill-list li {
    display: flex;
    align-items: stretch;
    gap: 0.5rem;
    padding: 0 0.5rem;
    border-bottom: 1px solid var(--border);
  }

  .skill-list li:last-child {
    border-bottom: none;
  }

  .skill-list li:hover {
    background: var(--row-hover);
  }

  .skill-list li.selected {
    background: var(--row-selected);
  }

  .skill-list li.active {
    outline: 1px solid var(--accent);
    outline-offset: -1px;
  }

  .skill-list input[type="checkbox"] {
    margin: auto 0;
    flex-shrink: 0;
  }

  .skill-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.15rem;
    padding: 0.55rem 0.25rem;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    min-width: 0;
  }

  .skill-list li.compact {
    align-items: center;
  }

  .skill-btn.compact {
    flex-direction: row;
    align-items: center;
    padding: 0.3rem 0.25rem;
  }

  .skill-btn.compact .name {
    font-weight: 500;
    font-size: 0.85rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .name {
    font-weight: 600;
    font-size: 0.9rem;
  }

  .variant {
    font-size: 0.75rem;
    color: var(--warn);
    font-weight: 500;
  }

  .desc {
    font-size: 0.8rem;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .context {
    font-size: 0.7rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    padding: 2rem;
  }
</style>
