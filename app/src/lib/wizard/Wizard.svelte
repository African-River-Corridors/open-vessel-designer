<script lang="ts">
  // The setup wizard (spec 0017, D2 and D3): five steps and a review, one question to a step. The logic is in
  // wizard.ts; this draws it. Choices, labels and limits come from the schema.
  import { tick } from "svelte";
  import { trapFocus } from "../focusTrap";
  import type { Choice, InputSpec, Schema } from "../api";
  import VesselClass from "../VesselClass.svelte";
  import { inputText, schemaText, t } from "../i18n.svelte";
  import ChoiceGroup from "./ChoiceGroup.svelte";
  import NumberField from "./NumberField.svelte";
  import { labelOf, problemText } from "./text";
  import {
    STEPS,
    canCreate,
    choicesSent,
    classChosen,
    inputsOnStep,
    leftToDefaults,
    optionName,
    problemsOnStep,
    riverOnly,
    type Answers,
    type Num,
    type StepId,
  } from "./wizard.ts";

  let { schema, answers, step: firstStep = "cargo", onCreate, onCancel }: {
    schema: Schema;
    /** The answers to start from: empty, or an example's. */
    answers: Answers;
    step?: StepId;
    /** Make the Design. Gives the reason when it could not, so the wizard stays open and shows it. */
    onCreate: (option: string, inputs: Record<string, unknown>) => Promise<string | null>;
    onCancel: () => void;
  } = $props();

  /* svelte-ignore state_referenced_locally */
  let a = $state<Answers>($state.snapshot(answers) as Answers);
  /* svelte-ignore state_referenced_locally */
  let step = $state<StepId>(firstStep);
  /** The steps where Next was pressed with something missing: only then are missing answers called errors. */
  let tried = $state<Partial<Record<StepId, boolean>>>({});
  let creating = $state(false);
  let failure = $state<string | null>(null);
  let heading = $state<HTMLElement>();

  const NAMES: Record<string, string> = {
    cargo: "app-wizard-step-cargo",
    area: "app-wizard-step-area",
    formation: "app-wizard-step-formation",
    hull: "app-wizard-step-hull",
    limits: "app-wizard-step-limits",
    review: "app-wizard-step-review",
  };
  const HELP: Record<string, string> = {
    cargo: "app-wizard-help-cargo",
    area: "app-wizard-help-area",
    formation: "app-wizard-help-formation",
    hull: "app-wizard-help-hull",
    limits: "app-wizard-help-limits",
    review: "app-wizard-help-review",
  };
  const AREA_HELP: Record<string, string> = {
    "river-only": "app-wizard-area-river-only",
    "coastal-and-river": "app-wizard-area-coastal-and-river",
  };

  const spec = (name: string): InputSpec | undefined => schema.inputs.find((i) => i.name === name);
  const choices = (name: string): Choice[] => spec(name)?.choices ?? [];
  const choiceLabel = (input: string, value: string | null): string | null => {
    const c = choices(input).find((x) => x.value === value);
    if (!c) return null;
    const name = schemaText.choice(input, c.value, c.label);
    return input === "cargo" && c.status === "development" ? t("app-cargo-development", { name }) : name;
  };
  /** A schema description is shown only when it is meant for people: a note for programmers holds `::`. */
  const forPeople = (d: string | undefined) => (d && !d.includes("::") ? d : undefined);
  const unitOf = (name: string) => {
    const u = spec(name)?.unit;
    return u && u !== "-" ? u : "";
  };

  let index = $derived(STEPS.indexOf(step));
  let problems = $derived(problemsOnStep(schema, a, step));
  let blocked = $derived(problems.some((p) => p.block));
  /** The messages of one input: a missing answer is shown only after Next was pressed. */
  const messagesOf = (input: string) =>
    problems
      .filter((p) => p.input === input && (p.kind !== "required" || tried[step]))
      .map((p) => ({ text: problemText(schema, p), block: p.block }));
  let reviewProblems = $derived(STEPS.flatMap((s) => problemsOnStep(schema, a, s)).filter((p) => p.block));
  let ok = $derived(canCreate(schema, a));
  let made = $derived(optionName(schema, a));

  async function go(to: StepId) {
    step = to;
    failure = null;
    await tick();
    heading?.focus();
  }
  function next() {
    if (blocked) {
      tried[step] = true;
      return;
    }
    go(STEPS[index + 1]);
  }
  const back = () => go(STEPS[index - 1]);

  async function create() {
    if (!ok) {
      tried.review = true;
      return;
    }
    creating = true;
    failure = null;
    try {
      failure = await onCreate(made, choicesSent(schema, a));
    } finally {
      creating = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape" && !creating) onCancel();
  }

  $effect(() => {
    heading?.focus();
  });

  /** Which step an input is answered on. */
  function stepOf(name: string): StepId {
    for (const s of STEPS.slice(0, 5)) if (inputsOnStep(schema, a, s).includes(name)) return s;
    if (name === "vessel_scheme" || name === "vessel_class") return "hull";
    return "limits";
  }

  const numberText = (name: string, n: unknown) => `${inputText(Number(n))}${unitOf(name) ? " " + unitOf(name) : ""}`;
  /** One typed choice as the review shows it. */
  function valueText(name: string, v: unknown): string {
    if (name === "formation") {
      const f = v as { kind: string; abreast?: number; in_line?: number };
      const label = choiceLabel("formation", f.kind) ?? f.kind;
      return f.kind === "pushed_convoy" ? `${label}: ${t("app-wizard-convoy-counts", { abreast: f.abreast ?? "", inLine: f.in_line ?? "" })}` : label;
    }
    if (typeof v === "number") return numberText(name, v);
    const c = choices(name).find((x) => x.value === v);
    return c ? (choiceLabel(name, String(v)) ?? String(v)) : String(v);
  }
  let typedRows = $derived(Object.entries(choicesSent(schema, a)).map(([name, v]) => ({ name, label: labelOf(schema, name), text: valueText(name, v), step: stepOf(name) })));

  /** The answers so far, one line a step, for the side list. */
  let sofar = $derived.by(() => {
    const rows = typedRows;
    return STEPS.slice(0, 5).map((s) => {
      const mine = rows.filter((r) => r.step === s);
      return { step: s, text: mine.length === 0 ? null : mine.map((r) => (s === "limits" || s === "hull" || s === "area" ? `${r.label}: ${r.text}` : r.text)).join("; ") };
    });
  });

  let heldClass = $derived(classChosen(schema, a));
  /** The Cargo groups of the moulded-depth rule, by the engine's group name (as in InputField). */
  const GROUP_IDS: Record<string, string> = {
    dry_bulk: "app-cargo-group-dry-bulk",
    liquid: "app-cargo-group-liquid",
    gas: "app-cargo-group-gas",
  };
  /** The wording of a hint under a limit: the default or rule that applies when the box is left empty. */
  function leftEmpty(s: InputSpec): string {
    if (s.name === "min_freeboard_m") return t("app-wizard-help-freeboard");
    if (s.default_rule) {
      // The schema's own rule text, as the input list shows it: on a Cargo that carries gas the depth follows the tank.
      const by = Object.entries(s.default_rule.ratios).map(([g, v]) => `${GROUP_IDS[g] ? t(GROUP_IDS[g]) : g} ${v}`).join(", ");
      return t("app-help-default-rule", { ratios: by });
    }
    if (s.default !== undefined && s.default !== null) return t("app-wizard-help-default", { value: numberText(s.name, s.default) });
    return "";
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="veil">
  <div class="dialog" role="dialog" aria-modal="true" use:trapFocus aria-labelledby="wizard-title" data-testid="wizard">
    <header>
      {#if step !== "review"}<p class="stepof" data-testid="wizard-step-of">{t("app-wizard-step-of", { n: index + 1, total: 5 })}</p>{/if}
      <h2 id="wizard-title" tabindex="-1" bind:this={heading} data-testid="wizard-heading">{t(NAMES[step])}</h2>
      <p class="plain" data-testid="wizard-help">{t(HELP[step])}</p>
    </header>

    <div class="grid">
      <div class="main">
        {#if step === "cargo"}
          <ChoiceGroup
            name="wiz-cargo"
            input="cargo"
            messages={messagesOf("cargo")}
            legend={labelOf(schema, "cargo")}
            options={choices("cargo").map((c) => ({ value: c.value, label: choiceLabel("cargo", c.value) ?? c.label, help: forPeople((c as Choice & { description?: string }).description) }))}
            value={a.cargo}
            onChange={(v) => (a.cargo = v)}
          />
        {:else if step === "area"}
          <ChoiceGroup
            name="wiz-area"
            input="service_area"
            messages={messagesOf("service_area")}
            legend={labelOf(schema, "service_area")}
            options={choices("service_area").map((c) => ({ value: c.value, label: choiceLabel("service_area", c.value) ?? c.label, help: AREA_HELP[c.value] ? t(AREA_HELP[c.value]) : undefined }))}
            value={a.serviceArea}
            onChange={(v) => (a.serviceArea = v)}
          />
          {#if riverOnly(a)}
            <p class="note">{t("app-wizard-river-limits-note")}</p>
            <NumberField id="draught_limit_m" label={labelOf(schema, "draught_limit_m")} unit={unitOf("draught_limit_m")} value={a.draughtLimit}
              onChange={(n: Num) => (a.draughtLimit = n)} messages={messagesOf("draught_limit_m")} />
            <NumberField id="air_draft_limit_m" label={labelOf(schema, "air_draft_limit_m")} unit={unitOf("air_draft_limit_m")} value={a.airDraftLimit}
              onChange={(n: Num) => (a.airDraftLimit = n)} messages={messagesOf("air_draft_limit_m")} />
          {/if}
        {:else if step === "formation"}
          <ChoiceGroup
            name="wiz-formation"
            input="formation"
            messages={messagesOf("formation")}
            legend={labelOf(schema, "formation")}
            options={choices("formation").map((c) => ({ value: c.value, label: choiceLabel("formation", c.value) ?? c.label }))}
            value={a.formation?.kind ?? null}
            onChange={(v) => (a.formation = { kind: v, abreast: a.formation?.abreast ?? null, inLine: a.formation?.inLine ?? null })}
          />
          {#if a.formation?.kind === "pushed_convoy"}
            <NumberField id="abreast" label={t("app-convoy-abreast")} unit={spec("formation")?.convoy?.abreast.unit ?? ""} value={a.formation.abreast}
              onChange={(n: Num) => a.formation && (a.formation.abreast = n)} messages={messagesOf("abreast")} />
            <NumberField id="in_line" label={t("app-convoy-in-line")} unit={spec("formation")?.convoy?.in_line.unit ?? ""} value={a.formation.inLine}
              onChange={(n: Num) => a.formation && (a.formation.inLine = n)} messages={messagesOf("in_line")} />
          {/if}
        {:else if step === "hull"}
          <ChoiceGroup
            name="wiz-hull-form"
            input="hull_form"
            messages={messagesOf("hull_form")}
            legend={labelOf(schema, "hull_form")}
            options={choices("hull_form").map((c) => ({ value: c.value, label: choiceLabel("hull_form", c.value) ?? c.label }))}
            value={a.hullForm}
            onChange={(v) => (a.hullForm = v)}
          />
          <div class="vessel">
            <VesselClass
              schemeSpec={spec("vessel_scheme")}
              classSpec={spec("vessel_class")}
              inputs={{
                vessel_scheme: { value: a.scheme, source: "typed" },
                vessel_class: { value: a.vesselClass, source: "typed" },
                formation: { value: { kind: a.formation?.kind }, source: "typed" },
              }}
              summary={null}
              messages={[]}
              byClassAtStart={a.hullMode === "class"}
              radioName="wizard-vc-mode"
              onMode={(byClass) => {
                a.hullMode = byClass ? "class" : "typed";
                if (!byClass) {
                  a.scheme = null;
                  a.vesselClass = null;
                }
              }}
              onEdit={(name, value) => {
                if (name === "vessel_scheme") {
                  a.scheme = (value as string | null) ?? null;
                  a.vesselClass = null;
                } else if (name === "vessel_class") a.vesselClass = value as string;
              }}
            />
            {#if heldClass}
              {@const h = heldClass as Choice & { length_max_m?: number; beam_max_m?: number }}
              <p class="note">{t("app-class-dimensions", { length: String(h.length_max_m ?? ""), beam: String(h.beam_max_m ?? "") })}</p>
            {/if}
            {#if messagesOf("vessel_scheme").length || messagesOf("vessel_class").length}
              {#each [...messagesOf("vessel_scheme"), ...messagesOf("vessel_class")] as m}<div class="msg error" role="alert" data-testid="message-vessel">{m.text}</div>{/each}
            {/if}
          </div>
          {#if a.hullMode === "typed"}
            <NumberField id="length_overall_m" label={labelOf(schema, "length_overall_m")} unit={unitOf("length_overall_m")} value={a.length}
              onChange={(n: Num) => (a.length = n)} messages={messagesOf("length_overall_m")} />
            <NumberField id="beam_m" label={labelOf(schema, "beam_m")} unit={unitOf("beam_m")} value={a.beam}
              onChange={(n: Num) => (a.beam = n)} messages={messagesOf("beam_m")} />
          {/if}
        {:else if step === "limits"}
          {#each inputsOnStep(schema, a, "limits") as name (name)}
            {@const s = spec(name)}
            {#if s}
              <NumberField id={name} label={labelOf(schema, name)} unit={unitOf(name)} value={a.own[name] ?? null}
                onChange={(n: Num) => (a.own[name] = n)} messages={messagesOf(name)} help={leftEmpty(s)} />
            {/if}
          {/each}
        {:else}
          <div class="field">
            <label for="wiz-option-name">{t("app-wizard-option-name")}</label>
            <input id="wiz-option-name" data-testid="input-option-name" type="text" class="name" value={a.optionName ?? made} oninput={(e) => (a.optionName = e.currentTarget.value)} />
          </div>
          <h3>{t("app-wizard-your-choices")}</h3>
          <ul class="rows" data-testid="wizard-choices">
            {#each typedRows as r (r.name)}
              <li data-input={r.name} data-testid="choice-row-{r.name}">
                <span class="k">{r.label}</span><span class="v">{r.text}</span>
                <button type="button" class="link" aria-label={t("app-wizard-change-input", { label: r.label })} data-testid="change-{r.name}" onclick={() => go(r.step)}>{t("app-wizard-change")}</button>
              </li>
            {/each}
          </ul>
          <p class="defaults" data-testid="wizard-defaults">{t("app-wizard-defaults-count", { count: leftToDefaults(schema, a), profile: schema.profile })}</p>
          {#if reviewProblems.length}
            <ul class="stops" data-testid="wizard-stops">
              {#each reviewProblems as p}
                <li class="msg error" data-testid="stop-{p.input}">{problemText(schema, p)} <button type="button" class="link" data-testid="stop-go-{p.input}" onclick={() => go(p.step)}>{t("app-wizard-go-to-step", { step: t(NAMES[p.step]) })}</button></li>
              {/each}
            </ul>
          {/if}
          {#if failure}<div class="msg error" role="alert" data-testid="wizard-failure">{failure}</div>{/if}
        {/if}
      </div>

      {#if step !== "review"}
        <aside aria-label={t("app-wizard-so-far")}>
          <h3>{t("app-wizard-so-far")}</h3>
          <ol class="sofar" data-testid="wizard-sofar">
            {#each sofar as s (s.step)}
              <li class:current={s.step === step} data-testid="sofar-{s.step}"><span class="k">{t(NAMES[s.step])}</span><span class="v">{s.text ?? t("app-wizard-not-chosen")}</span></li>
            {/each}
          </ol>
        </aside>
      {/if}
    </div>

    <footer>
      <button type="button" data-testid="wizard-cancel" onclick={onCancel} disabled={creating}>{t("app-wizard-cancel")}</button>
      <span class="spacer"></span>
      {#if index > 0}<button type="button" data-testid="wizard-back" onclick={back} disabled={creating}>{t("app-wizard-back")}</button>{/if}
      {#if step === "review"}
        <button type="button" class="primary" data-testid="wizard-create" aria-disabled={!ok || creating} onclick={create}>{t("app-wizard-create")}</button>
      {:else}
        <button type="button" class="primary" data-testid="wizard-next" aria-disabled={blocked} onclick={next}>{t("app-wizard-next")}</button>
      {/if}
    </footer>
  </div>
</div>

<style>
  .veil {
    position: fixed; inset: 0; z-index: 10; display: flex; align-items: center; justify-content: center;
    background: color-mix(in srgb, var(--blueprint-deep, #1a3753) 55%, transparent);
  }
  .dialog {
    background: var(--surface); color: var(--ink); border-top: 4px solid var(--gilt-mark); border-radius: var(--radius-lg, 6px);
    padding: 20px 26px; width: min(860px, calc(100vw - 32px)); max-height: calc(100vh - 32px); box-sizing: border-box;
    display: flex; flex-direction: column; box-shadow: 0 8px 30px color-mix(in srgb, var(--blueprint-deep, #1a3753) 40%, transparent);
  }
  header { flex: none; }
  .stepof { margin: 0; font-size: var(--fs-12); color: var(--ink-3); text-transform: uppercase; letter-spacing: 0.06em; }
  h2 { font-size: 22px; margin: 2px 0 4px; }
  h2:focus-visible { outline-offset: 4px; }
  h3 { font-size: var(--fs-14); margin: var(--sp-3) 0 var(--sp-1); }
  .plain { margin: 0 0 var(--sp-3); font-size: var(--fs-14); color: var(--ink-2); }
  .grid { flex: 1; min-height: 0; overflow: auto; display: grid; grid-template-columns: minmax(0, 1fr) 15em; gap: var(--sp-4); padding: 2px; }
  .main { min-width: 0; }
  aside { border-left: var(--lw-thin) solid var(--rule); padding-left: var(--sp-3); font-size: var(--fs-13); }
  aside h3 { margin-top: 0; }
  .sofar { list-style: none; margin: 0; padding: 0; }
  .sofar li { margin: 0 0 var(--sp-2); }
  .sofar li.current .k { color: var(--ink); font-weight: 600; }
  .k { display: block; font-size: var(--fs-12); color: var(--ink-2); }
  .v { display: block; color: var(--ink); }
  .note { font-size: var(--fs-13); color: var(--ink-2); margin: var(--sp-3) 0 0; }
  .vessel { margin-top: var(--sp-3); }
  .field { margin: 0 0 var(--sp-3); }
  .field label { display: block; font-size: var(--fs-14); margin-bottom: 2px; }
  input.name { width: min(24em, 100%); padding: 5px 8px; font: inherit; font-size: var(--fs-16); box-sizing: border-box; }
  .rows { list-style: none; margin: 0; padding: 0; border-top: var(--lw-thin) solid var(--rule); }
  .rows li { display: grid; grid-template-columns: 14em minmax(0, 1fr) auto; gap: var(--sp-2); align-items: baseline; padding: 5px 0; border-bottom: var(--lw-thin) solid var(--rule); font-size: var(--fs-14); }
  .rows .k { display: inline; font-size: var(--fs-13); }
  .rows .v { display: inline; font-variant-numeric: tabular-nums; }
  .defaults { font-size: var(--fs-13); color: var(--ink-2); }
  .stops { list-style: none; margin: var(--sp-2) 0; padding: 0; }
  .msg { font-size: var(--fs-13); margin-top: 3px; }
  .msg.error { color: var(--fail); }
  .msg.error::before { content: "✕ " / ""; }
  .link { background: none; border: none; color: var(--accent); text-decoration: underline; padding: 0; cursor: pointer; font-size: var(--fs-13); }
  .link:hover:not(:disabled) { background: none; }
  footer { flex: none; display: flex; gap: var(--sp-2); align-items: center; border-top: var(--lw-thin) solid var(--rule); margin-top: var(--sp-3); padding-top: var(--sp-3); }
  footer .spacer { flex: 1; }
  footer button { font-size: var(--fs-15, var(--fs-14)); padding: 5px 16px; }
  button.primary { background: var(--accent); color: var(--accent-ink); border-color: var(--accent); }
  button.primary:hover:not(:disabled) { background: var(--accent); filter: brightness(1.12); }
  button.primary[aria-disabled="true"] { background: var(--surface-2); color: var(--ink-3); border-color: var(--rule-strong); cursor: not-allowed; filter: none; }
  @media (max-width: 720px) { .grid { grid-template-columns: minmax(0, 1fr); } aside { border-left: 0; padding-left: 0; } }
</style>
