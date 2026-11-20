<script lang="ts">
  // The Summary view (issue #1081, spec 0017 D7): three questions answered with numbers and texts the
  // evaluation already holds. summary.ts picks the lines; nothing is calculated here, and each number is
  // formatted as the Outputs panel formats it (metrics.ts).
  import { focusInput } from "../focus";
  import { damageRun } from "../damageRun.svelte";
  import { MAIN, POWERING, TANKS, exact, show, type Metric } from "../metrics";
  import { checkName, listParts, parts, say, slug, t, type TextIds } from "../i18n.svelte";
  import { scopeCounts, shortTitle, summarise, type Blocker } from "../summary";
  import { vesselDims } from "../vessel";
  import VesselPicture from "../VesselPicture.svelte";
  import { summaryView } from "../summaryView.svelte";

  let { results, message, blockers, inputs, ids, option, calculating = false, drawings = null, onShowDrawing = () => {}, onRunDamage, onCancelDamage, onShowInput, onShowCheck, onShowRule = () => {} }: {
    /** The side elevation and plan sheets of the shown results, as the Drawings view shows them; null without results. */
    drawings?: { side: string | null; plan: string | null } | null;
    /** Open a drawing in the Drawings view. */
    onShowDrawing?: (view: "side" | "plan") => void;
    /** Issue #1096: a new Design is being calculated for the first time; there is nothing of it to show yet. */
    calculating?: boolean;
    /** Issue #1090: open the Rule checks view at a Rule's row, or at its top for `null`. */
    onShowRule?: (title: string | null) => void;
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
  /** The principal dimensions the pictures' text names; null leaves the Option's name alone. */
  let dims = $derived(vesselDims(results));
  /** The scope's two counts, from the result (issue: the Summary's scope line). */
  let counts = $derived(scopeCounts(results));
  const pictureLabel = (view: "side" | "plan") =>
    view === "side"
      ? dims ? t("app-summary-vessel-side", { option, ...dims }) : t("app-summary-vessel-side-name", { option })
      : dims ? t("app-summary-vessel-plan", { option, ...dims }) : t("app-summary-vessel-plan-name", { option });
  let mark = $derived(MARKS[verdict.kind]);
</script>

<section class="summary" data-testid="summary" aria-label={t("app-summary-label")}>
  {#if calculating}
    <h4>{t("app-summary-works")}</h4>
    <!-- Issue #1096: right after Create or Open, say plainly that the Design is being calculated. -->
    <p class="verdict pending" data-testid="summary-calculating">{t("app-summary-calculating")}</p>
    <!-- The picture's place: no drawing of the last Design while this one is calculated. -->
    <p class="vessel-pending" data-testid="summary-vessel-pending">{t("app-summary-vessel-pending")}</p>
  {:else}
  <!-- Issue #1097: the three questions in two columns where the view is wide enough, so the first answer is one page. -->
  <div class="cols">
  <div class="q works" data-testid="summary-works">
  <h4>{t("app-summary-works")}</h4>
  <p class="verdict {mark.tone}" data-testid="summary-verdict" data-kind={verdict.kind}>
    <span class="tag"><span aria-hidden="true">{mark.mark}</span> {t(mark.label)}</span>
    {#if verdict.kind === "met"}
      {t("app-summary-met")}
    {:else if verdict.kind === "not-met"}
      {t("app-summary-not-met-count", { count: verdict.count })}
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
  {#if results && drawings?.side}
    <!-- What did I make? The side elevation, and the plan under it where there is room. Above the damage lines,
         so nothing moves it when damage stability ends. -->
    <div class="vessel" data-testid="summary-vessel">
      <div class="pic side"><VesselPicture svg={drawings.side} label={pictureLabel("side")} hint={t("app-summary-vessel-open-side")} testid="summary-vessel-side" onOpen={() => onShowDrawing("side")} /></div>
      {#if drawings.plan}<div class="pic plan"><VesselPicture svg={drawings.plan} label={pictureLabel("plan")} hint={t("app-summary-vessel-open-plan")} testid="summary-vessel-plan" onOpen={() => onShowDrawing("plan")} /></div>{/if}
    </div>
  {/if}
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
  {#if verdict.kind === "not-met"}
    <!-- Issue #1090: one failed Rule per line, the first few, each with a link to its row. -->
    <ul class="failed" data-testid="summary-failed">
      {#each verdict.shown as f (f.title)}
        {@const name = checkName(f.title, ids)}
        <li data-testid="summary-failed-rule" data-rule={f.title}>
          <span class="name" title={name}>{shortTitle(name)}</span>
          {#if f.check}
            <a href={"#check-" + slug(f.check)} data-testid="summary-failed-link" data-to="check" onclick={(e) => { e.preventDefault(); onShowCheck(f.check!); }}>{t("app-summary-show-check")}</a>
          {:else}
            <a href="#rule-checks" data-testid="summary-failed-link" data-to="rule" onclick={(e) => { e.preventDefault(); onShowRule(f.title); }}>{t("app-summary-show-rule")}</a>
          {/if}
        </li>
      {/each}
    </ul>
    {#if verdict.more > 0}
      <p class="more" data-testid="summary-failed-more">{t("app-summary-more-rules", { count: verdict.more })}
        <a href="#rule-checks" data-testid="summary-open-rules" onclick={(e) => { e.preventDefault(); onShowRule(null); }}>{t("app-summary-open-rules")}</a></p>
    {/if}
  {/if}
  {#if summary.scope}
    <!-- The scope is reference text, not the answer: one line with its two counts, the sentence on opening. The
         line always shows the count of blocking Rules not evaluated. Open or closed is kept for the session. -->
    <details class="scope" data-testid="summary-scope-box" open={summaryView.scopeOpen} ontoggle={(e) => (summaryView.scopeOpen = e.currentTarget.open)}>
      <summary data-testid="summary-scope-line">{#if counts}{t("app-summary-scope-counts", { sets: counts.ruleSets, notEvaluated: counts.notEvaluated })}{#if counts.notRun > 0}{" "}{t("app-summary-scope-not-run", { count: counts.notRun })}{/if}{" "}{/if}<span class="show">{t(summaryView.scopeOpen ? "app-summary-scope-hide" : "app-summary-scope-show")}</span></summary>
      <p data-testid="summary-scope">{say(summary.scope, ids)}</p>
    </details>
  {/if}

  </div>
  <div class="q got" data-testid="summary-got">
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

  </div>
  <div class="q next" data-testid="summary-next">
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
            <a href={"#check-" + slug(w.name)} data-testid="summary-show-check" onclick={(e) => { e.preventDefault(); onShowCheck(w.name); }}>{t("app-summary-show-check")}</a>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
  <p data-testid="summary-input-counts">
    {t("app-summary-input-counts", { typed: summary.inputs.typed, defaults: summary.inputs.defaults })}
    <a href="#input-panel" data-testid="summary-open-input" onclick={(e) => { e.preventDefault(); onShowInput(); }}>{t("app-summary-open-input")}</a>
  </p>
  </div>
  </div>
  {/if}
</section>

<style>
  /* A calm page of text and numbers: no chart, no new colour. It scrolls inside its own box. */
  .summary { height: 100%; overflow: auto; box-sizing: border-box; padding-right: var(--sp-2); font-size: var(--fs-14); line-height: var(--lh-ui); max-width: 72em; container-type: size; }
  .q + .q { margin-top: var(--sp-4); }
  @container (min-width: 46em) {
    .cols { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); grid-template-areas: "works got" "next got"; column-gap: var(--sp-5); align-items: start; }
    .works { grid-area: works; }
    .got { grid-area: got; }
    .next { grid-area: next; }
    .q + .q { margin-top: 0; }
    .next { margin-top: var(--sp-4); }
  }
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
  .failed { list-style: none; margin: 0 0 var(--sp-2); padding: 0; }
  .failed li { padding: 3px 0; border-bottom: var(--lw-thin) solid var(--rule); }
  .failed .name { margin-right: var(--sp-2); }
  .more { color: var(--ink-2); }
  .verdict.pending { color: var(--accent); }
  .vessel-pending { color: var(--ink-3); }
  /* Fixed heights: a redraw never moves the numbers below. */
  .vessel { display: flex; flex-direction: column; gap: var(--sp-1); margin-bottom: 6px; }
  /* The side elevation always; the plan under it only where the Summary is tall and wide enough, so the verdict,
     the numbers and the picture fit a 1280 x 800 window. The Summary's box is a size container: its size, not its
     content, decides. */
  .pic.side { height: 120px; }
  .pic.plan { height: 96px; display: none; }
  .scope summary { cursor: pointer; color: var(--ink-2); font-size: var(--fs-13); }
  .scope .show { color: var(--accent); text-decoration: underline; white-space: nowrap; }
  .scope p { margin-top: var(--sp-1); }
  @container (min-height: 600px) and (min-width: 30em) { .pic.plan { display: block; } }
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
