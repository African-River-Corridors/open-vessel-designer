<script lang="ts">
  import { num, say, t, type TextIds } from "../i18n.svelte";
  import HullView from "../HullView.svelte";
  import { outcomeTag } from "../ruleOutcomes";
  import type { DamageViews } from "../api";
  import { conditionsOf, damageNotes, damageSets, damageStatus, governingCase, passes, secondsBefore, secondsLeft, wholeSeconds, type DamageCase, type DamageResults } from "../damage";
  import { damageRun } from "../damageRun.svelte";

  // The damage cases of the Option, as the engine gives them (the `damage`, `damage_by_rule` and
  // `marpol_damage` blocks of the results), the governing case of each loading condition as the Rule
  // Trace shows it, the intermediate stages where the Rule Set has them, and the damage drawing drawn in
  // Rust. The panel computes nothing. Where there are no damage cases, it gives the reason the trace gives.
  // Issue #924: damage runs on demand. The status line says whether it was run for these inputs, and
  // offers the run (with the case count and, once a run has been timed on this machine, the time it
  // takes), or shows the run's progress and Cancel. The panel computes nothing: the status and the case
  // count are the engine's, the time left is worked out in damage.ts from the progress it reports.
  let { results, ids, drawings, option, onRun, onCancel }: {
    results: DamageResults;
    ids: TextIds | null;
    drawings: DamageViews | null | undefined;
    /** The Option shown, to tell whether the run under way is its own. */
    option: string;
    onRun: () => void;
    onCancel: () => void;
  } = $props();

  const status = $derived(damageStatus(results as { damage_status?: unknown; damage_cases?: unknown }));
  const run = $derived(damageRun.active?.option === option ? damageRun.active : null);
  const left = $derived(run ? secondsLeft(run.done, run.total, run.lastAt - run.startedAt) : null);
  const before = $derived(secondsBefore(status.cases, damageRun.secondsPerCase));
  const STATUS_TEXT: Record<string, string> = {
    run: "app-damage-status-run",
    "not run": "app-damage-status-not-run",
    "not applicable": "app-damage-status-not-applicable",
    "not evaluated": "app-damage-status-not-evaluated",
  };

  const trace = $derived(results.rule_trace ?? null);
  const sets = $derived(damageSets(results));
  const notes = $derived(damageNotes(trace));
  /** Engine text (a case, a condition, a reason) in the Display Language. */
  const tr = (text: string) => say(text, ids);

  let setIndex = $state(0);
  let conditionPick = $state<Record<string, string>>({});
  let casePick = $state.raw<DamageCase | null>(null);

  const set = $derived(sets[Math.min(setIndex, sets.length - 1)]);
  const conditions = $derived(set ? conditionsOf(set) : []);
  const condition = $derived(set ? (conditionPick[set.id] !== undefined && conditions.includes(conditionPick[set.id]) ? conditionPick[set.id] : (conditions[0] ?? "")) : "");
  const cases = $derived(set ? set.cases.filter((c) => (c.condition ?? "") === condition) : []);
  const governing = $derived(set ? governingCase(set, condition, trace) : null);
  /** The case whose stages are shown: the one picked in this condition, or else the governing one. */
  const shown = $derived<DamageCase | null>((casePick && cases.includes(casePick) ? casePick : null) ?? governing ?? cases[0] ?? null);
  /** The sheet of a set that names a governing case (the gas path's `damage` block, or a `damage_by_rule` set): one per condition. */
  const sheet = $derived.by(() => {
    if (!set || set.engineGoverning.length === 0) return undefined;
    const key = set.id === "damage" ? undefined : set.id.startsWith("by-rule:") ? set.id.slice("by-rule:".length) : null;
    if (key === null || (key === undefined && !set.ruleSet)) return undefined;
    return (drawings?.sheets ?? []).find((s) => s.set === key && (s.condition ?? "") === condition);
  });

  const FLOATS: Record<string, string> = { yes: "app-damage-floats-yes", sinks: "app-damage-floats-sinks", "no equilibrium": "app-damage-floats-none" };
  const floats = (v: string) => (FLOATS[v] ? t(FLOATS[v]) : v);
  /** A number to d places, with no sign on a value that rounds to zero. */
  function f(v: number | null | undefined, d: number): string {
    if (v === null || v === undefined || !Number.isFinite(v)) return "";
    return num(Number(v.toFixed(d)) === 0 ? 0 : v, d);
  }
  /** The results hold nothing for a case that has no floating position. */
  const floating = (c: { floats: string }) => c.floats === "yes";
  const stageLabel = (fraction: number) => t("app-damage-stage-of", { percent: num(fraction * 100, 0) });
</script>

<div class="damage" data-testid="damage-stability">
  {#if status.status}
    <div class="runline" data-testid="damage-run-panel">
      <span class="label">{t("app-damage-status-label")}</span>
      <span class="tag {status.status === 'run' ? 'ok' : 'open'}" data-testid="damage-status" data-status={status.status}>{t(STATUS_TEXT[status.status])}</span>
      {#if status.cases !== null}<span class="dim" data-testid="damage-case-count">{t("app-damage-cases-count", { count: status.cases })}</span>{/if}
      {#if run}
        <progress aria-label={t("app-damage-progress-label")} value={run.done} max={run.total} data-testid="damage-progress" data-done={run.done} data-total={run.total}></progress>
        <span data-testid="damage-count">{t("app-damage-progress", { done: run.done, total: run.total })}</span>
        {#if left !== null}<span class="dim" data-testid="damage-eta">{t("app-damage-left", { seconds: wholeSeconds(left) })}</span>{/if}
        <button type="button" data-testid="damage-cancel" onclick={onCancel}>{t("app-damage-cancel")}</button>
      {:else if status.status === "not run"}
        {#if before !== null}<span class="dim" data-testid="damage-estimate">{t("app-damage-estimate", { seconds: wholeSeconds(before) })}</span>{/if}
        <button type="button" data-testid="damage-run" onclick={onRun}>{t("app-damage-run")}</button>
      {/if}
    </div>
  {/if}
  {#if !set}
    {#if status.status === "not run"}
      <p class="note" data-testid="damage-not-run">{t("app-damage-not-run-for-inputs")}</p>
    {:else}
      <p class="note" data-testid="damage-none">{t("app-damage-none")}</p>
    {/if}
    {#if notes.length > 0}
      <ul class="notes" data-testid="damage-not-evaluated">
        {#each notes as n, i (i)}
          <li data-outcome={n.outcome}>
            <span class="tag {outcomeTag(n.outcome).tone}">{t(outcomeTag(n.outcome).label ?? "app-rules-not-evaluated")}</span>
            <strong>{n.title}</strong> <span class="id">{n.rule} {n.sets.join(", ")}</span>
            {#if n.gap}<span class="why" data-testid="damage-reason">{tr(n.gap)}</span>{/if}
          </li>
        {/each}
      </ul>
    {/if}
  {:else}
    <div class="bar">
      {#if sets.length > 1}
        <div class="chips" role="tablist" aria-label={t("app-damage-rule-sets")}>
          {#each sets as s, i (s.id)}
            <button type="button" role="tab" aria-selected={i === setIndex} class:on={i === setIndex} data-testid="damage-set" onclick={() => { setIndex = i; casePick = null; }}>{s.ruleSet ?? t(s.id === "marpol" ? "app-damage-set-marpol" : "app-damage-set-adn")}</button>
          {/each}
        </div>
      {:else}
        <span class="setname" data-testid="damage-set-name">{set.ruleSet ?? t(set.id === "marpol" ? "app-damage-set-marpol" : "app-damage-set-adn")}</span>
      {/if}
      {#if set.method}<span class="dim" data-testid="damage-method">{tr(set.method)}</span>{/if}
      {#if conditions.length > 1}
        <div class="chips" role="tablist" aria-label={t("app-damage-conditions")}>
          {#each conditions as c (c)}
            <button type="button" role="tab" aria-selected={c === condition} class:on={c === condition} data-testid="damage-condition" onclick={() => { conditionPick = { ...conditionPick, [set.id]: c }; casePick = null; }}>{tr(c)}</button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="grid">
      <div class="cases" data-testid="damage-cases-body">
        <table data-testid="damage-cases">
          <thead>
            <tr>
              <th>{t("app-damage-col-case")}</th>
              <th>{t("app-damage-col-result")}</th>
              <th>{t("app-damage-col-floats")}</th>
              <th class="num">{t("app-damage-col-heel")}<span class="unit">&nbsp;{t("app-damage-unit-deg")}</span></th>
              <th class="num">{t("app-damage-col-trim")}<span class="unit">&nbsp;{t("app-damage-unit-deg")}</span></th>
              <th class="num">{t("app-damage-col-draught")}<span class="unit">&nbsp;{t("app-damage-unit-m")}</span></th>
              <th class="num">{t("app-damage-col-gz")}<span class="unit">&nbsp;{t("app-damage-unit-m")}</span></th>
              <th class="num">{t("app-damage-col-range")}<span class="unit">&nbsp;{t("app-damage-unit-deg")}</span></th>
              <th class="num">{t("app-damage-col-area")}<span class="unit">&nbsp;{t("app-damage-unit-m-rad")}</span></th>
              <th>{t("app-damage-col-flooded")}</th>
            </tr>
          </thead>
          <tbody>
            {#each cases as c, i (i)}
              {@const ok = passes(set, c, trace)}
              {@const isGoverning = c === governing}
              <tr class:sel={c === shown} data-testid="damage-case-row" data-ok={ok} data-governing={isGoverning} data-case={tr(c.name)}>
                <th scope="row">
                  <button type="button" class="pick" onclick={() => (casePick = c)}>{tr(c.name)}</button>
                  {#if isGoverning}<span class="mark" title={t("app-damage-governing-hint")}>{t("app-rules-governing")}</span>{/if}
                </th>
                <td><span class="tag {ok ? 'ok' : 'fail'}"><span aria-hidden="true">{ok ? "✓" : "✕"}</span> {t(ok ? "app-pass" : "app-fail")}</span></td>
                <td>{floats(c.floats)}</td>
                <td class="num" title={String(c.heel_deg)}>{floating(c) ? f(c.heel_deg, 1) : ""}</td>
                <td class="num" title={String(c.trim_deg)}>{floating(c) ? f(c.trim_deg, 2) : ""}</td>
                <td class="num" title={String(c.draught_mid_m)}>{floating(c) ? f(c.draught_mid_m, 2) : ""}</td>
                <td class="num" title={String(c.max_gz_m)}>{floating(c) ? f(c.max_gz_m, 2) : ""}</td>
                <td class="num" title={String(c.range_end_deg)}>{floating(c) ? f(c.range_end_deg, 1) : ""}</td>
                <td class="num" title={String(c.area_m_rad)}>{floating(c) ? f(c.area_m_rad, 3) : ""}</td>
                <td class="flooded">{c.flooded.join(", ")}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <div class="side">
        <div class="drawing" data-testid="damage-drawing">
          {#if sheet}
            <p class="caption" data-testid="damage-drawing-case">{t("app-damage-drawing-of", { case: tr(sheet.case ?? "") })}</p>
            {#key sheet.stem}
              <div class="hull"><HullView svg={sheet.svg} label={t("app-view-damage")} /></div>
            {/key}
          {:else if drawings?.message}
            <p class="note">{say(drawings.message, ids)}</p>
          {:else}
            <p class="note" data-testid="damage-no-drawing">{t("app-damage-no-drawing")}</p>
          {/if}
        </div>
        {#if shown && shown.stages && shown.stages.length > 0}
          <div class="stages" data-testid="damage-stages">
            <h4>{t("app-damage-stages", { case: tr(shown.name) })}</h4>
            <table>
              <thead>
                <tr>
                  <th>{t("app-damage-col-stage")}</th>
                  <th>{t("app-damage-col-result")}</th>
                  <th>{t("app-damage-col-floats")}</th>
                  <th class="num">{t("app-damage-col-heel")}<span class="unit">&nbsp;{t("app-damage-unit-deg")}</span></th>
                  <th class="num">{t("app-damage-col-gz")}<span class="unit">&nbsp;{t("app-damage-unit-m")}</span></th>
                  <th class="num">{t("app-damage-col-area")}<span class="unit">&nbsp;{t("app-damage-unit-m-rad")}</span></th>
                </tr>
              </thead>
              <tbody>
                {#each shown.stages as s (s.fraction)}
                  <tr data-testid="damage-stage-row" data-ok={s.ok}>
                    <th scope="row">{stageLabel(s.fraction)}</th>
                    <td><span class="tag {s.ok ? 'ok' : 'fail'}"><span aria-hidden="true">{s.ok ? "✓" : "✕"}</span> {t(s.ok ? "app-pass" : "app-fail")}</span></td>
                    <td>{floats(s.floats)}</td>
                    <td class="num" title={String(s.heel_deg)}>{s.floats === "yes" ? f(s.heel_deg, 1) : ""}</td>
                    <td class="num" title={String(s.max_gz_m)}>{s.floats === "yes" ? f(s.max_gz_m, 2) : ""}</td>
                    <td class="num" title={String(s.area_m_rad)}>{s.floats === "yes" ? f(s.area_m_rad, 3) : ""}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    </div>
    {#if notes.length > 0}
      <ul class="notes small" data-testid="damage-not-evaluated">
        {#each notes as n, i (i)}
          <li data-outcome={n.outcome}>
            <span class="tag {outcomeTag(n.outcome).tone}">{t(outcomeTag(n.outcome).label ?? "app-rules-not-evaluated")}</span>
            <strong>{n.title}</strong>
            {#if n.gap}<span class="why">{tr(n.gap)}</span>{/if}
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .damage { height: 100%; display: flex; flex-direction: column; gap: var(--sp-2); min-height: 0; }
  .note { color: var(--ink-3); margin: 0; }
  .runline { flex: none; display: flex; flex-wrap: wrap; align-items: center; gap: var(--sp-2) var(--sp-3); }
  .runline .label { font-weight: 600; }
  .runline progress { width: 12em; height: 0.8em; }
  .bar { flex: none; display: flex; flex-wrap: wrap; align-items: center; gap: var(--sp-2) var(--sp-4); }
  .setname { font-weight: 600; }
  .dim { color: var(--ink-3); font-size: var(--fs-12, 12px); }
  .chips { display: flex; flex-wrap: wrap; gap: var(--sp-1); }
  .chips button { border: var(--lw-thin) solid var(--rule-strong); border-radius: 12px; background: none; padding: 1px var(--sp-3); font-size: var(--fs-13); color: var(--ink); cursor: pointer; }
  .chips button:hover { background: var(--surface-2); }
  .chips button.on { background: var(--accent-soft); border-color: var(--accent); }
  .grid { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr); gap: var(--sp-3); }
  .cases { min-height: 0; overflow: auto; border: var(--lw-thin) solid var(--rule); }
  .side { min-height: 0; display: flex; flex-direction: column; gap: var(--sp-2); }
  .drawing { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: var(--sp-1); }
  .hull { flex: 1; min-height: 0; }
  .caption { flex: none; margin: 0; font-size: var(--fs-12, 12px); color: var(--ink-3); }
  .stages { flex: none; max-height: 40%; overflow: auto; border: var(--lw-thin) solid var(--rule); }
  .stages h4 { margin: 0; padding: var(--sp-1) 10px; font: 600 var(--fs-13) var(--font-ui); background: var(--surface-2); }
  table { border-collapse: collapse; width: 100%; font-size: var(--fs-13); }
  thead th { position: sticky; top: 0; background: var(--surface-2); color: var(--ink); text-align: left; font-weight: 600; padding: var(--sp-1) 10px; border-bottom: var(--lw-thick) solid var(--ink); white-space: nowrap; }
  td, tbody th { padding: var(--sp-1) 10px; vertical-align: top; text-align: left; border-bottom: var(--lw-thin) solid var(--rule); }
  tbody th { font-weight: 600; }
  tr.sel { background: var(--accent-soft); }
  .flooded { min-width: 8em; }
  .pick { background: none; border: 0; padding: 0; font: inherit; font-weight: 600; color: var(--ink); text-align: left; cursor: pointer; }
  .num { text-align: right; font-variant-numeric: tabular-nums; white-space: nowrap; }
  th.num { text-align: right; }
  .unit { color: var(--ink-3); font-weight: 400; }
  .mark { display: block; font: 600 var(--fs-11) var(--font-ui); color: var(--accent-ink); margin-top: 2px; }
  .tag { font: 600 var(--fs-11) var(--font-ui); padding: 1px 6px; border-radius: var(--radius); white-space: nowrap; display: inline-block; }
  .tag.ok { background: var(--pass-soft); color: var(--pass); }
  .tag.fail { background: var(--fail-soft); color: var(--fail); }
  .tag.open { background: transparent; color: var(--ink-2); border: var(--lw-thin) dashed var(--ink-3); }
  .tag.plain { background: var(--surface-2); color: var(--ink-3); }
  .notes { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--sp-1); font-size: var(--fs-13); }
  .notes.small { flex: none; max-height: 22%; overflow: auto; font-size: var(--fs-12, 12px); }
  .id { font: var(--fs-11) var(--font-mono); color: var(--ink-3); }
  .why { display: block; color: var(--ink-2); }
</style>
