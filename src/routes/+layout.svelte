<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { initApp } from "$lib/stores/app.svelte";

  let { children } = $props();

  onMount(() => {
    initApp();
  });
</script>

{@render children()}
