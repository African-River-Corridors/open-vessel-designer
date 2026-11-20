<script lang="ts">
  // The sign "Calculating…" (issues #1082, #1096). Drawn in one place at a time: the top of the Input panel, or
  // the views' tab row while the Input panel is hidden. Always in the page there, so a screen reader hears the
  // text once, politely, when it arrives.
  import { t } from "./i18n.svelte";

  let { on }: { on: boolean } = $props();
</script>

<span class="calculating" role="status" data-testid="calculating">{on ? t("app-calculating") : ""}</span>

<style>
  .calculating { color: var(--accent); font-size: var(--fs-13); font-weight: 600; white-space: nowrap; }
</style>
