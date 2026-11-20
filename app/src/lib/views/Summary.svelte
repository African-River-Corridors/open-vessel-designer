<script lang="ts">
  // The Summary view (issue #1081, spec 0017 D7): three questions answered with numbers and texts the
  // evaluation already holds. summary.ts picks the lines; nothing is calculated here, and each number is
  // formatted as the Outputs panel formats it (metrics.ts).
  import { focusInput } from "../focus";
  import { damageRun } from "../damageRun.svelte";
  import { MAIN, POWERING, TANKS, exact, show, type Metric } from "../metrics";
  import { checkName, listParts, parts, say, slug, t, type TextIds } from "../i18n.svelte";
  import { summarise, type Blocker } from "../summary";

  let { results, message, blockers, inputs, ids, option, calculating = false, onRunDamage, onCancelDamage, onShowInput, onShowCheck }: {
    /** Issue #1096: a new Design is being calculated for the first time; there is nothing of it to show yet. */
    calculating?: boolean;
    /** The shown Option's fresh results; null where the Option is not calculated. */
    results: Record<string, unknown> | null;
    /** The engine's message where the Option has no results and no input to fix. */
    message: string | null;
    blockers: Blocker[];
    /** The stored inputs of the Option, each with its source. */
    inputs: Record<string, { source: string }>;
    ids: TextIds | null;
    option: string;
    onRunDamage: () => void;
    onCancelDamage: () => void;
    /** Show the Input panel (and so focus it). */
    onShowInput: () => void;
    /** Show a check in the Checks panel. */
    onShowCheck: (name: string) => void;
  } = $props();

  const METRICS: Metric[] = [...MAIN, ...TANKS, ...POWERING];
  const metricOf = (label: string) => METRICS.find((m) => m.label === label)!;

  let summary = $derived(summarise({ results, message, blockers, inputs }));
  let running = $derived(damageRun.active?.option === option ? damageRun.active : null);
  const MARKS = {
    met: { mark: "✓", label: "app-pass", tone: "ok" },
    "not-met": { mark: "✕", label: "app-fail", tone: "fail" },
    "not-calculated": { mark: "!", label: "app-rules-warn", tone: "warn" },
  } as const;
  const DAMAGE_TEXT: Record<string, string> = {
    "not run": "app-summary-damage-not-run",
    "not applicable": "app-summary-damage-not-applicable",
    "not evaluated": "app-summary-damage-not-evaluated",
  };
  let verdict = $derived(summary.verdict);
  let mark = $derived(MARKS[verdict.kind]);
</script>

<section class="summary" data-testid="summary" aria-label={t("app-summary-label")}>
  <h4>{t("app-summary-works")}</h4>
  {#if calculating}
    <!-- Issue #1096: right after Create or Open, say plainly that the Design is being calculated. -->
    <p class="verdict pending" data-testid="summary-calculating">{t("app-summary-calculating")}</p>
  {:else}
  <p class="verdict {mark.tone}" data-testid="summary-verdict" data-kind={verdict.kind}>
    <span class="tag"><span aria-hidden="true">{mark.mark}</span> {t(mark.label)}</span>
    {#if verdict.kind === "met"}
      {t("app-summary-met")}
    {:else if verdict.kind === "not-met"}
      {t("app-summary-not-met", { rules: verdict.titles.map((x) => checkName(x, ids)).join("; ") })}
    {:else if verdict.inputs.length > 0}
      <!-- The link of the "Not recalculated" line: it focuses the input to fix. -->
      {#each parts("app-summary-not-calculated-fix", ["inputs"]) as p}{#if typeof p === "string"}{p}{:else}{#each listParts(verdict.inputs.map((b) => b.label)) as q}{#if typeof q === "string"}{q}{:else}{@const b = verdict.inputs[Number(q.arg)]}<a
          href={"#" + b.name}
          onclick={(e) => { e.preventDefault(); focusInput(b.name); }}>{b.label}</a>{/if}{/each}{/if}{/each}
    {:else if verdict.message}
      {t("app-summary-not-calculated-because", { message: verdict.message })}
    {:else}
      {t("app-summary-not-calculated")}
    {/if}
  </p>
  {#if summary.scope}<p class="scope" data-testid="summary-scope">{say(summary.scope, ids)}</p>{/if}
  {#if running}
    <!-- Issue #1096: damage stability can still turn the verdict, so say so while it runs. -->
    <p class="provisional" data-testid="summary-provisional">{t("app-summary-may-change")}</p>
    <p class="damage" data-testid="summary-damage" data-status="running">
      {t("app-damage-running")}: {t("app-damage-progress", { done: running.done, total: running.total })}
      <button type="button" data-testid="summary-damage-cancel" onclick={onCancelDamage}>{t("app-damage-cancel")}</button>
    </p>
  {:else if summary.damage.status}
    <p class="damage" data-testid="summary-damage" data-status={summary.damage.status}>
      {#if summary.damage.status === "run"}
        {summary.damage.cases === null ? t("app-summary-damage-run") : t("app-summary-damage-run-cases", { count: summary.damage.cases })}
      {:else}
        {t(DAMAGE_TEXT[summary.damage.status])}
      {/if}
      {#if summary.damage.status === "not run"}
        <button type="button" data-testid="summary-damage-run" onclick={onRunDamage}>{t("app-damage-run")}</button>
      {/if}
    </p>
  {/if}

  <h4>{t("app-summary-got")}</h4>
  {#if summary.rows.length === 0 || !results}
    <p class="note" data-testid="summary-no-results">{t("app-summary-no-results")}</p>
  {:else}
    <table data-testid="summary-numbers">
      <tbody>
        {#each summary.rows as r (r.label)}
          {@const m = metricOf(r.label)}
          <tr data-testid="summary-number">
            <th>{t(m.label)}</th>
            <td class="num" class:text={m.quantity === "text"} title={exact(m, r.value)}><span>{show(m, r.value, ids)}</span>{#if r.value !== null && r.value !== undefined && m.unit}<span class="unit">&nbsp;{m.unit}</span>{/if}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <h4>{t("app-summary-next")}</h4>
  {#if results}
    {#if summary.warnings.length === 0}
      <p class="note" data-testid="summary-no-warnings">{t("app-summary-no-warnings")}</p>
    {:else}
      <ul class="warnings">
        {#each summary.warnings as w (w.name)}
          <li data-testid="summary-warning">
            <span class="tag"><span aria-hidden="true">!</span> {t("app-rules-warn")}</span>
            <strong>{checkName(w.name, ids)}</strong>
            {#if w.message && w.message !== "OK"}<span class="msg">{say(w.message, ids)}</span>{/if}
            <a href={"#check-" + slug(w.name)} onclick={(e) => { e.preventDefault(); onShowCheck(w.name); }}>{t("app-summary-show-check")}</a>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
  <p data-testid="summary-input-counts">
    {t("app-summary-input-counts", { typed: summary.inputs.typed, defaults: summary.inputs.defaults })}
    <a href="#input-panel" data-testid="summary-open-input" onclick={(e) => { e.preventDefault(); onShowInput(); }}>{t("app-summary-open-input")}</a>
  </p>
  {/if}
</section>

<style>
  /* A calm page of text and numbers: no chart, no new colour. It scrolls inside its own box. */
  .summary { height: 100%; overflow: auto; box-sizing: border-box; padding-right: var(--sp-2); font-size: var(--fs-14); line-height: var(--lh-ui); max-width: 56em; }
  h4 { font: 600 var(--fs-12) var(--font-ui); text-transform: uppercase; letter-spacing: 0.08em; color: var(--ink-2); background: var(--surface-2); border-bottom: var(--lw-thin) solid var(--rule); margin: var(--sp-4) 0 var(--sp-2); padding: var(--sp-1) var(--sp-2); }
  h4:first-child { margin-top: 0; }
  p { margin: var(--sp-2) 0; overflow-wrap: anywhere; }
  .scope { color: var(--ink-2); font-size: var(--fs-13); }
  .note { color: var(--ink-3); }
  .tag { font: 600 var(--fs-11) var(--font-ui); padding: 1px 6px; border-radius: var(--radius); margin-right: var(--sp-1); white-space: nowrap; }
  .ok .tag { background: var(--pass-soft); color: var(--pass); }
  .fail .tag { background: var(--fail-soft); color: var(--fail); }
  .warn .tag, .warnings .tag { background: var(--warn-soft); color: var(--warn); }
  .verdict { font-weight: 600; }
  .verdict.pending { color: var(--accent); }
  .provisional { color: var(--warn); font-weight: 600; }
  table { border-collapse: collapse; font-size: var(--fs-14); }
  tr { border-bottom: var(--lw-thin) solid var(--rule); }
  th { text-align: left; font-weight: 400; color: var(--ink-2); padding: 2px var(--sp-4) 2px var(--sp-1); white-space: nowrap; }
  td { padding: 2px var(--sp-2); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  td.text { text-align: left; white-space: normal; }
  .unit { color: var(--ink-3); }
  .warnings { list-style: none; margin: 0 0 var(--sp-3); padding: 0; }
  .warnings li { padding: 3px 0; border-bottom: var(--lw-thin) solid var(--rule); }
  .msg { color: var(--ink-3); margin-left: var(--sp-1); }
  a { color: var(--accent); text-decoration: underline; cursor: pointer; }
  button { margin-left: var(--sp-2); }
</style>
