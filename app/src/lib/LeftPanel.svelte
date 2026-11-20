<script lang="ts">
  import type { OptionEvaluation, Schema, Snapshot } from "./api";
  import InputField from "./InputField.svelte";
  import { hiddenByEssentials, shownInputs } from "./inputFilter";
  import { inputsView, setInputsFilter, setInputsMode } from "./inputsView.svelte";
  import VesselClass from "./VesselClass.svelte";
  import CalculatingSign from "./CalculatingSign.svelte";
  import { schemaText, t } from "./i18n.svelte";

  // The left panel. The groups, their order, the labels, the choices and the ranges all come from
  // the schema. An input is drawn when the Option holds it, so the panel needs no rule for which
  // inputs a Cargo Containment uses: Rust sends only those that apply.
  let { snap, schema, evaluation, generation, sign, onPick, onEdit, onReset, onFormation, onImportOffsets, onOption }: {
    snap: Snapshot | null;
    /** Issue #1096: the "Calculating…" sign, on or off; null when it is drawn elsewhere (the panel is hidden). */
    sign: boolean | null;
    schema: Schema | null;
    evaluation: OptionEvaluation | undefined;
    /** Changes when the fields must be drawn afresh. */
    generation: number;
    onPick: (name: string) => void;
    onEdit: (name: string, value: unknown) => void;
    onReset: (name: string) => void;
    onFormation: (kind: string, abreast: number | null, inLine: number | null) => void;
    /** Issue #451: import an offsets CSV into the active Option. */
    onImportOffsets: () => void;
    onOption: (action: "add" | "duplicate" | "rename" | "delete", newName?: string) => void;
  } = $props();

  let option = $derived(snap?.design.options.find((o) => o.name === snap!.active_option));
  // Issue #1034: a river-only Option needs a number for its two river limits. A field is drawn afresh when this
  // changes, so the limit boxes show the moment the Service Area becomes river-only.
  let riverOnly = $derived(option?.inputs["service_area"]?.value === "river-only");
  // Issue #1084: Essentials draws fewer inputs, and text in the filter box draws the inputs it matches, in every
  // group. The choice of inputs is made in inputFilter.ts. See isShown (issues #925, #1034) for what the Option holds.
  let held = $derived(option?.inputs ?? {});
  let filtering = $derived(inputsView.query.trim() !== "");
  let drawn = $derived(
    shownInputs(schema?.inputs ?? [], held, riverOnly, inputsView.mode, inputsView.query, (i) => schemaText.input(i.name, i.label)),
  );
  let groups = $derived(
    (schema?.groups ?? []).map((g) => ({
      name: g,
      inputs: drawn.filter((i) => i.group === g && i.kind !== "vessel_scheme" && i.kind !== "vessel_class"), // drawn by VesselClass
    })).filter((g) => g.inputs.length > 0 || (g.name === VESSEL_CLASS_GROUP && vesselClassShown)),
  );
  // The Vessel Class group is drawn whole, by VesselClass, when either of its two inputs is drawn.
  let vesselClassShown = $derived((inputsView.mode === "all" && !filtering) || drawn.some((i) => i.kind === "vessel_scheme" || i.kind === "vessel_class"));
  let hiddenCount = $derived(hiddenByEssentials(schema?.inputs ?? [], held, riverOnly));
  let matchCount = $derived(drawn.length);
  const VESSEL_CLASS_GROUP = "Vessel Class";
  const specOf = (name: string) => schema?.inputs.find((i) => i.name === name);
  let classMessages = $derived([...messagesFor("vessel_scheme"), ...messagesFor("vessel_class")]);

  let renaming = $state(false);
  let renameText = $state("");
  let confirmDelete = $state(false);
  let only = $derived((snap?.design.options.length ?? 0) <= 1);

  function messagesFor(name: string) {
    return (evaluation?.messages ?? []).filter((m) => m.inputs.includes(name));
  }
  function startRename() {
    renameText = snap?.active_option ?? "";
    renaming = true;
    confirmDelete = false;
  }
  function endRename(ok: boolean) {
    renaming = false;
    if (ok && renameText.trim() && renameText.trim() !== snap?.active_option) onOption("rename", renameText);
  }
  // Optionally-open groups are display state: closed groups stay closed across edits.
  let closed = $state<Record<string, boolean>>({});
</script>

<div class="panel">
  {#if snap}
    <section class="options">
      <h2>{t("app-option")}</h2>
      <select data-testid="option-picker" aria-label={t("app-option-active")} value={snap.active_option} onchange={(e) => onPick(e.currentTarget.value)}>
        {#each snap.design.options as o (o.name)}
          <option value={o.name}>{o.name}</option>
        {/each}
      </select>
      <div class="buttons">
        <button data-testid="option-add" onclick={() => onOption("add")}>{t("app-option-add")}</button>
        <button onclick={() => onOption("duplicate")}>{t("app-option-duplicate")}</button>
        <button onclick={startRename}>{t("app-option-rename")}</button>
        <button onclick={() => (confirmDelete = true)} disabled={only} title={only ? t("app-option-delete-last") : ""}>{t("app-option-delete")}</button>
      </div>
      {#if renaming}
        <div class="inline">
          <input
            type="text"
            aria-label={t("app-option-new-name")}
            bind:value={renameText}
            onkeydown={(e) => { if (e.key === "Enter") endRename(true); if (e.key === "Escape") endRename(false); }}
          />
          <button onclick={() => endRename(true)}>{t("app-ok")}</button>
          <button onclick={() => endRename(false)}>{t("app-cancel")}</button>
        </div>
      {/if}
      {#if confirmDelete}
        <div class="inline confirm" role="alert">
          {t("app-option-delete-confirm", { option: snap.active_option })}
          <button onclick={() => { confirmDelete = false; onOption("delete"); }}>{t("app-option-delete")}</button>
          <button onclick={() => (confirmDelete = false)}>{t("app-cancel")}</button>
        </div>
      {/if}
    </section>

    <section class="view">
      <div class="moderow">
        <div class="mode" role="group" aria-label={t("app-inputs-mode")}>
          <button type="button" data-testid="mode-essentials" aria-pressed={inputsView.mode === "essentials"} onclick={() => setInputsMode("essentials")}>{t("app-inputs-essentials")}</button>
          <button type="button" data-testid="mode-all" aria-pressed={inputsView.mode === "all"} onclick={() => setInputsMode("all")}>{t("app-inputs-all")}</button>
        </div>
        <!-- Issue #1096: the sign beside the switch, where the eyes are while typing. -->
        {#if sign !== null}<CalculatingSign on={sign} />{/if}
      </div>
      <div class="inline filter">
        <input
          type="text"
          data-testid="input-filter"
          aria-label={t("app-inputs-filter")}
          placeholder={t("app-inputs-filter-placeholder")}
          value={inputsView.query}
          oninput={(e) => setInputsFilter(e.currentTarget.value)}
          onkeydown={(e) => { if (e.key === "Escape") setInputsFilter(""); }}
        />
        {#if filtering}<button type="button" data-testid="input-filter-clear" onclick={() => setInputsFilter("")}>{t("app-inputs-filter-clear")}</button>{/if}
      </div>
      <p class="count" role="status" data-testid="input-filter-count">{filtering ? t("app-inputs-filter-count", { count: matchCount }) : ""}</p>
    </section>

    {#each groups as g (g.name)}
      <!-- While the filter has text, every group with a match is open; the open and closed state the designer left is kept. -->
      <details open={filtering || !closed[g.name]} ontoggle={(e) => { if (!filtering) closed[g.name] = !e.currentTarget.open; }}>
        <summary>{schemaText.group(g.name)}</summary>
        {#if schema?.group_notes?.[g.name]}<p class="gnote" data-testid="group-note">{schemaText.note(g.name, schema.group_notes[g.name])}</p>{/if}
        {#if g.name === VESSEL_CLASS_GROUP}
          <VesselClass
            schemeSpec={specOf("vessel_scheme")}
            classSpec={specOf("vessel_class")}
            inputs={option!.inputs}
            summary={evaluation?.vessel_class}
            messages={classMessages}
            onEdit={(name, v) => onEdit(name, v)}
          />
        {/if}
        {#each g.inputs as spec (`${spec.name}|${snap.active_option}|${generation}|${riverOnly}`)}
          <InputField
            {spec}
            stored={option!.inputs[spec.name] ?? { value: null, source: "typed" }}
            messages={messagesFor(spec.name)}
            onEdit={(v) => onEdit(spec.name, v)}
            onReset={() => onReset(spec.name)}
            designSpeed={option!.inputs["design_speed_kn"]?.value as number | undefined}
            {riverOnly}
            {onFormation}
            {onImportOffsets}
          />
        {/each}
      </details>
    {/each}
    {#if inputsView.mode === "essentials" && !filtering && hiddenCount > 0}
      <p class="more" data-testid="inputs-more">{t("app-inputs-more", { count: hiddenCount })} <button type="button" class="link" onclick={() => setInputsMode("all")}>{t("app-inputs-show-all")}</button></p>
    {/if}
  {/if}
</div>

<style>
  .panel { padding: 10px var(--sp-3); }
  h2 { font-family: var(--font-serif); font-size: var(--fs-16); font-weight: 600; color: var(--ink); margin: 10px 0 4px; }
  select, input[type="text"] { width: 100%; padding: 3px 5px; font: inherit; color: var(--ink); background-color: var(--surface); border: var(--lw-thin) solid var(--rule-strong); border-radius: var(--radius); box-sizing: border-box; }
  select { padding-right: var(--sp-5); }
  select:focus-visible, input[type="text"]:focus-visible { outline-offset: 0; }
  .buttons { display: flex; gap: var(--sp-1); margin-top: 6px; flex-wrap: wrap; }
  .buttons button, .inline button { font-size: var(--fs-13); }
  .inline { display: flex; gap: var(--sp-1); margin-top: 6px; align-items: center; font-size: var(--fs-13); }
  .confirm { background: var(--fail-soft); color: var(--fail); padding: var(--sp-1) 6px; }
  details { border-top: var(--lw-thin) solid var(--rule); margin-top: 10px; padding-top: var(--sp-1); }
  .view { margin-top: 10px; }
  .moderow { display: flex; flex-wrap: wrap; align-items: center; gap: var(--sp-1) var(--sp-3); }
  .mode { display: inline-flex; }
  .mode button { background: transparent; font-size: var(--fs-13); border-radius: 0; margin-left: -1px; }
  .mode button:first-child { border-radius: var(--radius) 0 0 var(--radius); margin-left: 0; }
  .mode button:last-child { border-radius: 0 var(--radius) var(--radius) 0; }
  .mode button[aria-pressed="true"],
  .mode button[aria-pressed="true"]:hover:not(:disabled) { background: var(--accent-soft); border-color: var(--accent); position: relative; z-index: 1; }
  .mode button:focus-visible { position: relative; z-index: 2; }
  .filter { margin-top: 6px; }
  .count { margin: 2px 0 0; font-size: var(--fs-12); color: var(--ink-3); min-height: 1em; }
  .more { margin: var(--sp-2) 0; font-size: var(--fs-12); color: var(--ink-3); }
  .link { background: none; border: none; color: var(--accent); text-decoration: underline; padding: 0; cursor: pointer; font-size: inherit; }
  .link:hover:not(:disabled) { background: none; }
  .gnote { margin: var(--sp-1) 0; font-size: var(--fs-12); color: var(--ink-3); }
  summary { font-family: var(--font-serif); font-weight: 600; color: var(--ink); font-size: var(--fs-16); cursor: pointer; }
</style>
