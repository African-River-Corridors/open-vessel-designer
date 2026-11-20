<script lang="ts">
  // One number box of the wizard. The box keeps what is typed; the wizard gets a number, `null` for an empty
  // box and `NaN` for text that is not a number. The messages come from the wizard's checks of the schema's limits.
  import { inputText, parseInput } from "../i18n.svelte";
  import type { Num } from "./wizard.ts";

  let { id, label, unit = "", value, onChange, messages = [], help = "" }: {
    id: string;
    label: string;
    unit?: string;
    value: Num;
    onChange: (n: Num) => void;
    /** Messages to show under the box: blocking ones are errors, the others warnings. */
    messages?: { text: string; block: boolean }[];
    help?: string;
  } = $props();

  /* svelte-ignore state_referenced_locally */
  let text = $state(value === null || Number.isNaN(value) ? "" : inputText(value));
  let invalid = $derived(messages.some((m) => m.block) && text !== "");
</script>

<div class="field" data-input={id}>
  <label for="wiz-{id}">{label}</label>
  {#if help}<div class="help" id="wiz-{id}-help">{help}</div>{/if}
  <div class="row">
    <input
      id="wiz-{id}"
      type="text"
      inputmode="decimal"
      class:invalid
      aria-invalid={invalid}
      aria-describedby={[help ? `wiz-${id}-help` : "", messages.length ? `wiz-${id}-msg` : ""].filter(Boolean).join(" ") || undefined}
      value={text}
      oninput={(e) => {
        text = e.currentTarget.value;
        onChange(text.trim() === "" ? null : parseInput(text));
      }}
    />
    <span class="unit">{unit && unit !== "-" ? unit : ""}</span>
  </div>
  {#if messages.length}
    <div id="wiz-{id}-msg">
      {#each messages as m}
        <div class="msg {m.block ? 'error' : 'warning'}" data-severity={m.block ? "error" : "warning"}>{m.text}</div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .field { margin: var(--sp-3) 0; }
  label { display: block; font-size: var(--fs-14); color: var(--ink); margin-bottom: 2px; }
  .help { font-size: var(--fs-12); color: var(--ink-3); margin-bottom: 4px; }
  .row { display: flex; align-items: center; gap: 6px; }
  input { width: 10em; padding: 5px 8px; font: inherit; font-size: var(--fs-16); text-align: right; font-variant-numeric: tabular-nums; box-sizing: border-box; }
  input.invalid { border-width: var(--lw-thick); border-color: var(--fail); padding: 4px 7px; }
  .unit { min-width: 2.2em; font-size: var(--fs-13); color: var(--ink-3); }
  .msg { font-size: var(--fs-13); margin-top: 3px; }
  .msg.error { color: var(--fail); }
  .msg.error::before { content: "✕ " / ""; }
  .msg.warning { color: var(--warn); }
  .msg.warning::before { content: "▲ " / ""; }
</style>
