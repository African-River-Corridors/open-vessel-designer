<script lang="ts">
  // A question with large clear choices: native radio buttons drawn as cards, so the arrow keys, Tab and Space work.
  let { name, input, legend, options, value, onChange, messages = [] }: {
    name: string;
    /** The schema input this answers; it names the test ids (`choice-<input>-<value>`). */
    input: string;
    legend: string;
    options: { value: string; label: string; help?: string }[];
    value: string | null;
    onChange: (v: string) => void;
    /** What is missing or wrong, said under the choices. */
    messages?: { text: string; block: boolean }[];
  } = $props();
</script>

<fieldset data-testid="choices-{input}">
  <legend>{legend}</legend>
  <div class="choices">
    {#each options as o (o.value)}
      <label class="choice" class:chosen={value === o.value} data-testid="choice-{input}-{o.value}">
        <input type="radio" {name} value={o.value} checked={value === o.value} onchange={() => onChange(o.value)} />
        <span class="text"><span class="name">{o.label}</span>{#if o.help}<span class="help">{o.help}</span>{/if}</span>
      </label>
    {/each}
  </div>
  {#each messages as m}<div class="msg error" role="alert" data-testid="message-{input}">{m.text}</div>{/each}
</fieldset>

<style>
  fieldset { border: 0; margin: 0; padding: 0; min-width: 0; }
  legend { padding: 0; margin-bottom: var(--sp-2); font-size: var(--fs-13); color: var(--ink-2); }
  .choices { display: grid; grid-template-columns: repeat(auto-fill, minmax(11em, 1fr)); gap: var(--sp-2); }
  .choice {
    display: flex; gap: var(--sp-2); align-items: flex-start; padding: var(--sp-2) var(--sp-3);
    background: var(--surface); border: var(--lw-thin) solid var(--rule-strong); border-radius: var(--radius); cursor: pointer;
  }
  .choice:hover { background: var(--surface-2); }
  .choice.chosen { background: var(--accent-soft); border-color: var(--accent); border-width: var(--lw-thick); padding: calc(var(--sp-2) - 1px) calc(var(--sp-3) - 1px); }
  .choice:focus-within { outline: var(--lw-thick) solid var(--accent); outline-offset: 2px; }
  input { margin: 4px 0 0; accent-color: var(--accent); }
  input:focus-visible { outline: none; }
  .text { display: flex; flex-direction: column; gap: 2px; }
  .name { font-size: var(--fs-16); font-weight: 500; }
  .help { font-size: var(--fs-12); color: var(--ink-2); }
  .msg { font-size: var(--fs-13); margin-top: var(--sp-2); }
  .msg.error { color: var(--fail); }
  .msg.error::before { content: "✕ " / ""; }
</style>
