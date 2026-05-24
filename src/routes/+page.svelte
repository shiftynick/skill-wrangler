<script lang="ts">
  import CopyPanel from "$lib/components/CopyPanel.svelte";
  import RootPicker from "$lib/components/RootPicker.svelte";
  import SkillPreview from "$lib/components/SkillPreview.svelte";
  import SkillTable from "$lib/components/SkillTable.svelte";
  import { uiState } from "$lib/stores/ui.svelte";
</script>

<div class="app">

  {#if uiState.error}
    <div class="error-banner" role="alert">{uiState.error}</div>
  {/if}

  <main class="layout">
    <aside class="sidebar">
      <RootPicker />
      <CopyPanel />
    </aside>

    <section class="main">
      <SkillTable />
    </section>

    <aside class="preview-pane">
      <SkillPreview />
    </aside>
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 1rem 1.25rem;
    gap: 0.75rem;
    overflow: hidden;
  }

  .layout {
    display: grid;
    grid-template-columns: 280px 1fr 320px;
    gap: 1rem;
    flex: 1;
    min-height: 0;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow-y: auto;
    min-height: 0;
  }

  .main {
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
  }

  .preview-pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  @media (max-width: 1100px) {
    .layout {
      grid-template-columns: 260px 1fr;
      grid-template-rows: 1fr auto;
    }

    .preview-pane {
      grid-column: 1 / -1;
      max-height: 220px;
    }
  }
</style>
