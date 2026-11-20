<script lang="ts">
  import { onMount, tick } from "svelte";
  import * as api from "./lib/api";
  import { Calculating } from "./lib/calculating.ts";
  import type { Evaluation, Schema, Snapshot, OptionEvaluation } from "./lib/api";
  import LeftPanel from "./lib/LeftPanel.svelte";
  import MainPanel from "./lib/MainPanel.svelte";
  import BottomPanel from "./lib/BottomPanel.svelte";
  import Notice from "./lib/Notice.svelte";
  import StartPage from "./lib/wizard/StartPage.svelte";
  import Wizard from "./lib/wizard/Wizard.svelte";
  import { answersFromInputs, emptyAnswers, type Answers, type StepId } from "./lib/wizard/wizard.ts";
  import SettingsDialog from "./lib/SettingsDialog.svelte";
  import MessageBar, { type BarMessage } from "./lib/MessageBar.svelte";
  import { loadPanels, savePanels } from "./lib/panels";
  import { list, say, schemaText, setInputLabels, t } from "./lib/i18n.svelte";
  import { damageRun } from "./lib/damageRun.svelte";
  import { showView } from "./lib/viewRequest.svelte";
  import { setInputsMode } from "./lib/inputsView.svelte";
  import { damageStatus, saveSecondsPerCase, secondsLeft, secondsPerCase, wholeSeconds } from "./lib/damage";

  // Display state only: the Design, its checks and its results all come from Rust.
  let snap = $state<Snapshot | null>(null);
  let schema = $state<Schema | null>(null);
  let evaluation = $state<Evaluation | null>(null);
  /** The last Option evaluation that had results, per Option name, shown "out of date" on a hard error. */
  let lastGood = $state<Record<string, OptionEvaluation>>({});
  let error = $state<string | null>(null);
  let bannerDismissed = $state(false);
  /** Migration notes the user dismissed, by their text (a rename gives a note a new text). */
  let notesDismissed = $state<string[]>([]);
  let generation = $state(0);
  /** The first-run notice: open on first start, and again from About. */
  let noticeOpen = $state(false);
  let noticeIsFirstRun = $state(false);
  /** The Start page (spec 0017, D1): open at first start, and from New. It replaces nothing until a path is chosen. */
  let startOpen = $state(false);
  /** The setup wizard, while it is open: the answers it starts from, and the step. */
  let wizard = $state<{ answers: Answers; step: StepId } | null>(null);
  /** Settings: Display Language, theme and Defaults Profile. */
  let settingsOpen = $state(false);
  /** The browser build: shows a label and a link to the desktop app. */
  const web = api.isWeb();

  let leftWidth = $state(340);
  let bottomHeight = $state(260);
  // Both side panels are shown unless the user hid one last time.
  const saved = loadPanels();
  let showInput = $state(saved.input);
  let showOutputs = $state(saved.outputs);
  $effect(() => savePanels({ input: showInput, outputs: showOutputs }));

  const EVALUATE_PAUSE_MS = 150;
  let pending: ReturnType<typeof setTimeout> | undefined;
  let queue: Array<{ send: () => Promise<Snapshot>; redraw: boolean }> = [];

  // Issue #1082 (spec 0017, D13): "Calculating…" from the moment an edit is sent until the new evaluation is
  // drawn, but only once the wait passes about 200 ms, so a quick result does not flash it.
  const CALCULATING_AFTER_MS = 200;
  let calculating = $state(false);
  const sign = new Calculating(CALCULATING_AFTER_MS, (on) => (calculating = on));
  const whileCalculating = <T,>(work: () => Promise<T>) => sign.during(work);

  async function guard<T>(work: () => Promise<T>): Promise<T | undefined> {
    try {
      error = null;
      return await work();
    } catch (e) {
      error = String(e);
      return undefined;
    }
  }

  async function runEvaluation() {
    const result = await api.evaluate();
    for (const o of result.options) {
      if (o.outcome.status === "results") lastGood[o.name] = o;
    }
    evaluation = result;
  }

  /** What the last damage run said when it did not leave results (cancelled, dropped, refused), for the message bar. */
  let damageNote = $state<string | null>(null);

  /**
   * Issue #924: run the damage cases of an Option, with progress, then evaluate again. Edits sent first, so
   * the run is for the inputs on screen. An edit during the run cancels it (Rust does that on the desktop,
   * the platform on the web): its results would be for inputs that are gone.
   */
  async function runDamage(option: string) {
    if (damageRun.active) return;
    damageNote = null;
    await guard(async () => {
      await flush();
      const started = performance.now();
      const outcome = evaluation?.options.find((o) => o.name === option)?.outcome;
      const cases = damageStatus(outcome?.status === "results" ? outcome.results : null).cases;
      damageRun.active = { option, done: 0, total: cases ?? 0, startedAt: started, baseAt: null, lastAt: started };
      try {
        const end = await api.runDamage(option, (p) => {
          const a = damageRun.active;
          if (!a || a.option !== p.option) return;
          a.done = p.done;
          a.total = p.total;
          a.lastAt = performance.now();
          a.baseAt ??= a.lastAt;
        });
        const last = damageRun.active;
        if (end.outcome === "done") {
          // The time per case of a whole run: the first progress event (the cases start) to the last.
          const per = last?.baseAt != null ? secondsPerCase(last.total, last.lastAt - last.baseAt) : null;
          if (per !== null) {
            damageRun.secondsPerCase = per;
            saveSecondsPerCase(per);
          }
        } else if (end.outcome === "failed") error = t("app-damage-did-not-run", { reason: end.message ?? "" });
        else damageNote = t(end.outcome === "cancelled" ? "app-damage-cancelled" : "app-damage-dropped");
      } finally {
        damageRun.active = null;
      }
      await runEvaluation();
    });
  }

  const cancelDamage = () => guard(api.cancelDamage);

  /** Take a new Design state: forget the last good results of a different Design. */
  async function adopt(next: Snapshot | null | undefined, newDesign = false, redraw = newDesign) {
    if (!next) return;
    if (newDesign) {
      showView("summary");
      lastGood = {};
      bannerDismissed = false;
      notesDismissed = [];
    }
    if (redraw) generation += 1;
    snap = next;
    await runEvaluation();
  }

  async function fileAction(action: () => Promise<Snapshot | null>, newDesign: boolean) {
    await guard(async () => {
      await flush();
      const next = await action();
      if (next) await adopt(next, newDesign);
    });
  }

  /** Send queued edits and evaluate, now. */
  async function flush() {
    clearTimeout(pending);
    pending = undefined;
    const work = queue;
    queue = [];
    if (work.length === 0) return;
    await whileCalculating(async () => {
      let last: Snapshot | undefined;
      for (const w of work) last = await w.send();
      await adopt(last, false, work.some((w) => w.redraw));
    });
  }

  /** An edit: send it after a pause in typing (about 150 ms), then evaluate the whole Design. */
  function edit(send: () => Promise<Snapshot>, redraw = false) {
    queue.push({ send, redraw });
    clearTimeout(pending);
    pending = setTimeout(() => guard(flush), EVALUATE_PAUSE_MS);
  }

  /** Option and profile commands: send what is queued, run the command, evaluate. */
  async function command(run: () => Promise<Snapshot | null>, refetchSchema = false) {
    await guard(async () => {
      await flush();
      const next = await run();
      if (!next) return;
      if (refetchSchema) schema = await api.inputSchema();
      if (schema) setInputLabels(Object.fromEntries(schema.inputs.map((i) => [i.name, i.label])));
      await adopt(next, false, true);
    });
  }

  function optionAction(action: "add" | "duplicate" | "rename" | "delete", newName?: string) {
    const name = snap?.active_option ?? "";
    if (action === "add") command(api.addOption);
    else if (action === "duplicate") command(() => api.duplicateOption(name));
    else if (action === "rename") command(() => api.renameOption(name, newName ?? ""));
    else command(() => api.deleteOption(name));
  }

  /** The Start page's own paths: each replaces the Design only when it succeeds, so a refusal leaves the page open. */
  async function startThen(action: () => Promise<Snapshot | null>) {
    const before = generation;
    await fileAction(action, true);
    if (generation !== before) startOpen = false;
  }

  /**
   * Create Design (spec 0017, D4 and D5): ask before unsaved work is discarded, make the Design from the typed
   * choices, evaluate, and start damage stability once when the Design says it has not been run. Gives the
   * reason when the choices were refused, so the wizard stays open and shows it.
   */
  async function createFromWizard(option: string, inputs: Record<string, unknown>): Promise<string | null> {
    try {
      await flush();
      if (snap?.dirty && !(await api.confirmDiscard())) return null;
      const next = await api.newDesignFrom(option, inputs);
      wizard = null;
      error = null;
      await adopt(next, true);
      // adopt() opened the Summary view for the new Design (#1081); the Input panel opens on Essentials.
      setInputsMode("essentials");
      const outcome = evaluation?.options.find((o) => o.name === next.active_option)?.outcome;
      if (outcome?.status === "results" && damageStatus(outcome.results).status === "not run") void runDamage(next.active_option);
      return null;
    } catch (e) {
      return String(e);
    }
  }

  function startWizard(from: Answers, step: StepId) {
    startOpen = false;
    wizard = { answers: from, step };
  }

  function onKey(e: KeyboardEvent) {
    if (!(e.metaKey || e.ctrlKey)) return;
    const k = e.key.toLowerCase();
    // New and Open start something new: not while a dialog is open over the app.
    const dialogOpen = noticeOpen || startOpen || wizard !== null || settingsOpen;
    if ((k === "o" || k === "n") && dialogOpen) {
      e.preventDefault(); // else the browser's own Open dialog (Ctrl+O) or a new window (Ctrl+N) opens over the dialog
      return;
    }
    if (k === "s") {
      e.preventDefault();
      fileAction(e.shiftKey ? api.saveAs : api.save, false);
    } else if (k === "o") {
      e.preventDefault();
      fileAction(api.openFile, true);
    } else if (k === "n") {
      e.preventDefault();
      startOpen = true;
    }
  }

  async function closeNotice() {
    noticeOpen = false;
    if (noticeIsFirstRun) {
      noticeIsFirstRun = false;
      await guard(api.acceptNotice);
    }
  }

  onMount(() => {
    guard(async () => {
      if (!(await api.noticeSeen())) {
        noticeIsFirstRun = true;
        noticeOpen = true;
      }
      schema = await api.inputSchema();
      setInputLabels(Object.fromEntries(schema.inputs.map((i) => [i.name, i.label])));
      // Listen before the first snapshot: a Design given to the app at launch is either in that
      // snapshot already or announced after the listener is in place (#671).
      await api.onOpenedFromOutside(
        () => fileAction(api.snapshot, true),
        (reason) => (error = reason),
      );
      const first = await api.snapshot();
      // A Design given at launch, or brought back from the last visit, is the one to show: no Start page.
      startOpen = first.path === null && !(await api.designRestored());
      await adopt(first, true);
    });
  });

  function drag(axis: "x" | "y") {
    return (down: PointerEvent) => {
      const target = down.currentTarget as HTMLElement;
      target.setPointerCapture(down.pointerId);
      const move = (e: PointerEvent) => {
        if (axis === "x") leftWidth = Math.min(Math.max(e.clientX, 200), window.innerWidth - 300);
        else bottomHeight = Math.min(Math.max(window.innerHeight - e.clientY, 100), window.innerHeight - 160);
      };
      const up = () => {
        target.removeEventListener("pointermove", move);
        target.removeEventListener("pointerup", up);
      };
      target.addEventListener("pointermove", move);
      target.addEventListener("pointerup", up);
    };
  }

  let active = $derived(snap?.active_option ?? "");
  /** The active Option's design speed, from the Design session's inputs. */
  let designSpeed = $derived.by(() => {
    const v = snap?.design.options.find((o) => o.name === active)?.inputs.max_speed_kn?.value;
    return typeof v === "number" ? v : null;
  });
  /** The active Option's stored inputs, for the Summary's count of typed and default inputs. */
  let activeInputs = $derived(snap?.design.options.find((o) => o.name === active)?.inputs ?? {});
  /** The Summary's links: show the Input panel, or a check of the Checks panel, and put the focus there. */
  async function showInputPanel() {
    showInput = true;
    await tick();
    document.querySelector<HTMLElement>("section.left")?.focus();
  }
  async function showCheck(name: string) {
    showOutputs = true;
    await tick();
    const row = document.querySelector<HTMLElement>(`[data-check="${CSS.escape(name)}"]`);
    row?.scrollIntoView({ block: "center" });
    row?.focus();
  }
  let activeEval = $derived(evaluation?.options.find((o) => o.name === active));
  /** Whether damage stability was run for the active Option's shown results (issue #924). */
  let activeStatus = $derived(activeEval?.outcome.status === "results" ? damageStatus(activeEval.outcome.results).status : null);
  /**
   * Every message the app shows, for the message bar, worked out from the state each time (issue
   * #436): an app error, the file's messages, the notes of a changed old file and the hash warning.
   * Errors stay until they are fixed; notes and the hash warning can be dismissed.
   */
  let messages = $derived.by((): BarMessage[] => {
    const out: BarMessage[] = [];
    if (error) out.push({ key: "error", severity: "error", text: error, dismiss: () => (error = null) });
    for (const m of evaluation?.file_messages ?? []) {
      out.push({ key: `file:${m.text}`, severity: m.severity === "error" ? "error" : "warning", text: say(m.text, null, m.msg) });
    }
    (snap?.migration_notes ?? []).forEach((note, i) => {
      if (notesDismissed.includes(note)) return;
      const dismiss = () => (notesDismissed = [...notesDismissed, note]);
      out.push({ key: `note:${note}`, severity: "note", text: say(note, null, snap?.migration_note_msgs?.[i]), dismiss });
    });
    // Issue #924: damage runs on demand. The line shows the run under way, or offers one.
    const run = damageRun.active;
    if (run) {
      const left = secondsLeft(run.done, run.total, run.lastAt - (run.baseAt ?? run.lastAt));
      const count = t("app-damage-progress", { done: run.done, total: run.total });
      out.push({
        key: "damage-run",
        severity: "note",
        text: `${t("app-damage-running")}: ${count}${left === null ? "" : `, ${t("app-damage-left", { seconds: wholeSeconds(left) })}`}`,
        progress: { done: run.done, total: run.total, label: t("app-damage-progress-label") },
        action: { label: t("app-damage-cancel"), run: cancelDamage },
      });
    } else if (activeStatus === "not run") {
      out.push({ key: "damage-not-run", severity: "note", text: t("app-damage-not-run"), action: { label: t("app-damage-run"), run: () => runDamage(active) } });
    }
    if (damageNote) out.push({ key: "damage-note", severity: "note", text: damageNote, dismiss: () => (damageNote = null) });
    if (snap && snap.hash_changed.length > 0 && !bannerDismissed) {
      out.push({ key: "hash-changed", severity: "note", text: t("app-hash-changed", { options: list(snap.hash_changed, "unit") }), dismiss: () => (bannerDismissed = true) });
    }
    return out;
  });

  /** The inputs of the active Option that have a hard error, with their schema labels. */
  let blockers = $derived.by(() => {
    const names = new Set<string>();
    for (const m of activeEval?.messages ?? []) if (m.severity === "error") m.inputs.forEach((n) => names.add(n));
    return [...names].map((name) => {
      const label = schema?.inputs.find((i) => i.name === name)?.label;
      return { name, label: label ? schemaText.input(name, label) : name };
    });
  });
</script>

<svelte:window onkeydown={onKey} />

{#if noticeOpen}<Notice onClose={closeNotice} />{/if}
{#if startOpen && !noticeOpen}<StartPage examples={api.designExamples}
  onWizard={() => startWizard(emptyAnswers(), "cargo")}
  onExample={(ex) => startWizard(answersFromInputs(ex.inputs), "review")}
  onOpen={() => startThen(api.openFile)}
  onSkip={() => startThen(api.newFile)}
  onCancel={() => (startOpen = false)} />{/if}
{#if schema && wizard}<Wizard {schema} answers={wizard.answers} step={wizard.step} onCreate={createFromWizard} onCancel={() => (wizard = null)} />{/if}
{#if settingsOpen}<SettingsDialog profile={snap?.profile ?? null} onClose={() => (settingsOpen = false)}
  onProfile={(a) => command(a === "load" ? api.loadProfile : api.useBuiltInProfile, true)} />{/if}

<!-- The Start page and the wizard are dialogs over the app: while one is open, Tab stays inside it. -->
<div class="shell" inert={(startOpen && !noticeOpen) || wizard !== null}>
  <header>
    <!-- The 32 px OVD mark, drawn in the text colour so it reads in both themes. -->
    <svg class="mark" viewBox="0 0 64 64" width="32" height="32" aria-hidden="true" focusable="false">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M5.5 34 H58.5" stroke-width="3" />
        <path d="M32 43.5 C24.5 43.5 19 37 18.8 14.4" stroke-width="3" />
        <path d="M32 48 C37.8 45.8 44.6 34 45.6 12.8" stroke-width="3" />
        <path d="M32 48 H18 Q10 48 10 40 V16 M32 48 H46 Q54 48 54 40 V16" stroke-width="5" />
      </g>
    </svg>
    <span class="brand">OVD</span>
    <button onclick={() => (startOpen = true)}>{t("app-new")}</button>
    <button onclick={() => fileAction(api.openFile, true)}>{t("app-open")}</button>
    <button onclick={() => fileAction(api.save, false)}>{t("app-save")}</button>
    <button onclick={() => fileAction(api.saveAs, false)}>{t("app-save-as")}</button>
    <button onclick={() => (noticeOpen = true)}>{t("app-about")}</button>
    {#if web}<span class="web-badge" data-testid="web-label">{t("app-web-version")}</span> <a href="/ovd#install">{t("app-download-desktop")}</a>{/if}
    <span class="status">
      {#if snap}
        {@const fileText = snap.dirty ? t("app-file-unsaved", { file: snap.file_name }) : snap.file_name}
        {@const profileText = t("app-header-profile", { profile: snap.profile })}
        {@const engineText = t("app-engine-version", { version: snap.engine_version })}
        <span class="meta file" title={fileText}>{fileText}</span>
        <span class="meta" title={profileText}>{profileText}</span>
        <span class="meta" data-testid="engine-version" title={engineText}>{engineText}</span>
      {/if}
    </span>
    <button type="button" aria-label={t(showInput ? "app-input-panel-hide" : "app-input-panel-show")} aria-pressed={showInput} onclick={() => (showInput = !showInput)}>{t("app-input-panel")}</button>
    <button type="button" aria-label={t(showOutputs ? "app-outputs-panel-hide" : "app-outputs-panel-show")} aria-pressed={showOutputs} onclick={() => (showOutputs = !showOutputs)}>{t("app-outputs-panel")}</button>
    <button type="button" data-testid="settings" aria-haspopup="dialog" onclick={() => (settingsOpen = true)}>{t("app-settings")}</button>
  </header>

  <!-- A line of its own under the header, so the sign takes no width from the file name and never covers a
       button. Always in the page, so a screen reader hears the text once, politely, when it arrives. -->
  <div class="signline"><span class="calculating" role="status" data-testid="calculating">{calculating ? t("app-calculating") : ""}</span></div>

  <MessageBar {messages} />

  <div class="body" style:grid-template-columns={showInput ? `${leftWidth}px 6px 1fr` : "0 0 1fr"} style:grid-template-rows={showOutputs ? `1fr 6px ${bottomHeight}px` : "1fr 0 0"}>
    <section class="left" hidden={!showInput} tabindex="-1"><LeftPanel {snap} {schema} {generation} evaluation={activeEval}
      onPick={(name) => command(() => api.setActiveOption(name))}
      onEdit={(name, value) => {
        const option = active;
        // A number being typed stays in its box; a choice, switch or "none" redraws the form.
        edit(() => api.setInput(option, name, value), typeof value !== "number");
      }}
      onReset={(name) => { const option = active; edit(() => api.resetInput(option, name), true); }}
      onImportOffsets={() => { const option = active; command(() => api.importOffsets(option)); }}
      onFormation={(kind, abreast, inLine) => {
        const option = active;
        edit(() => api.setFormation(option, kind, abreast, inLine), false);
      }}
      onOption={optionAction} /></section>
    <div class="handle v" hidden={!showInput} role="separator" aria-orientation="vertical" onpointerdown={drag("x")}></div>
    <section class="main"><MainPanel {evaluation} {active} {lastGood} {blockers} {designSpeed} inputs={activeInputs} onRunDamage={() => runDamage(active)} onCancelDamage={cancelDamage} onShowInput={showInputPanel} onShowCheck={showCheck} /></section>
    <div class="handle h" hidden={!showOutputs} role="separator" aria-orientation="horizontal" onpointerdown={drag("y")}></div>
    <section class="bottom" hidden={!showOutputs}><BottomPanel {evaluation} {active} {lastGood} {blockers} onActivate={(name) => command(() => api.setActiveOption(name))} /></section>
  </div>
</div>

<style>
  .shell { display: flex; flex-direction: column; height: 100vh; }
  header {
    display: flex; flex-wrap: wrap; gap: var(--sp-2); align-items: center; box-sizing: border-box;
    min-height: 40px; padding: 0 var(--sp-3);
    background: var(--paper); color: var(--ink);
    border-bottom: var(--lw-thick) solid var(--gilt-mark);
  }
  .mark { color: var(--ink); flex: none; }
  .brand { font-weight: 600; font-size: var(--fs-14); letter-spacing: 0.08em; margin-right: var(--sp-2); }
  header button { background: var(--surface); }
  header button:hover:not(:disabled) { background: var(--surface-2); }
  header button[aria-pressed] { background: transparent; }
  header button[aria-pressed="true"],
  header button[aria-pressed="true"]:hover:not(:disabled) { background: var(--accent-soft); border-color: var(--accent); }
  /* Basis 0: a wrapping row breaks lines on each item's hypothetical size, so the texts never push a button
     or add a row. The span is one text line high and clips the rest: a text that does not fit on the line
     wraps below it and is gone whole, never cut to a stub. The file name comes first, so it goes last. */
  .status {
    flex: 1 1 0; min-width: 0; display: flex; flex-wrap: wrap; align-items: center; align-content: flex-start;
    justify-content: flex-start; gap: var(--sp-2); height: 1.5em; line-height: 1.5em; overflow: clip;
  }
  /* No `overflow: hidden` on a text-only element: WebKit's WebDriver reads its text as empty (see the design note). */
  .meta { color: var(--ink-3); font-size: var(--fs-12); flex: 0 0 auto; white-space: nowrap; }
  /* A thin line under the header, the sign at its right. It keeps its height, so the panels never jump. */
  .signline { flex: none; display: flex; justify-content: flex-end; align-items: center; box-sizing: border-box; height: 18px; padding: 0 var(--sp-3); }
  .calculating { color: var(--ink-3); font-size: var(--fs-12); line-height: 18px; white-space: nowrap; }
  .meta.file { flex: 0 1 auto; min-width: 0; overflow: clip; text-overflow: ellipsis; font-family: var(--font-mono); }
  header :global(.web-badge) {
    font-size: var(--fs-11); text-transform: uppercase; letter-spacing: 0.08em; padding: 1px var(--sp-2);
    background: var(--surface-2); color: var(--ink-2); border: var(--lw-thin) solid var(--rule-strong); border-radius: var(--radius);
  }
  header :global(a) { color: var(--accent); font-size: var(--fs-13); }
  .body { flex: 1; display: grid; min-height: 0; }
  .body > section { overflow: auto; min-width: 0; min-height: 0; }
  .left { grid-column: 1; grid-row: 1 / 4; }
  .handle.v { grid-column: 2; grid-row: 1 / 4; cursor: col-resize; background: var(--rule); }
  .main { grid-column: 3; grid-row: 1; }
  .body > .main { overflow: hidden; }
  .handle.h { grid-column: 3; grid-row: 2; cursor: row-resize; background: var(--rule); }
  .bottom { grid-column: 3; grid-row: 3; }
  .body > [hidden] { display: none; }
  .handle:hover { background: var(--gilt-mark); }
</style>
