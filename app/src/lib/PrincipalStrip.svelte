<script lang="ts">
  import { language, t } from "./i18n.svelte";
  import { principalItems } from "./principal";

  // The principal characteristics of the shown Option (issue #889). Every value is read from the run; a value the
  // run does not hold shows "not evaluated". It is one compact block that wraps, so no view loses its fit.
  let { results }: { results: unknown } = $props();
  let items = $derived(principalItems(results, language().numbers, t("app-principal-not-evaluated")));
</script>

<dl class="strip" data-testid="principal-strip" aria-label={t("app-principal-title")}>
  {#each items as i (i.id)}
    <div class="item" data-row={i.id} class:missing={!i.evaluated}>
      <dt>{t(i.labelId)}</dt>
      <dd data-testid="principal-value">{i.text}</dd>
    </div>
  {/each}
</dl>

<style>
  .strip { flex: none; display: flex; flex-wrap: wrap; gap: 2px var(--sp-4); margin: 0; padding: var(--sp-1) var(--sp-4); background: var(--surface); border-bottom: var(--lw-thin) solid var(--rule); font-size: var(--fs-13); }
  .item { display: flex; gap: var(--sp-1); align-items: baseline; }
  dt { color: var(--ink-3); }
  dt::after { content: ":"; }
  dd { margin: 0; color: var(--ink); font-weight: 600; font-variant-numeric: tabular-nums; }
  .missing dd { color: var(--ink-3); font-weight: 400; font-style: italic; }
</style>
