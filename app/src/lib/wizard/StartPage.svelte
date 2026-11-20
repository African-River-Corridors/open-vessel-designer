<script lang="ts">
  // The Start page (spec 0017, D1): three choices and one link. It opens over the app; nothing is replaced until
  // a path is chosen, so Cancel (or Escape) leaves the Design as it is.
  import type { DesignExample } from "../api";
  import { t } from "../i18n.svelte";

  let { examples, onWizard, onExample, onOpen, onSkip, onCancel }: {
    /** The examples of the core's example call, read when the list is opened. */
    examples: () => Promise<DesignExample[]>;
    onWizard: () => void;
    onExample: (example: DesignExample) => void;
    onOpen: () => void;
    onSkip: () => void;
    onCancel: () => void;
  } = $props();

  let view = $state<"start" | "examples">("start");
  let list = $state<DesignExample[] | null>(null);
  let failure = $state<string | null>(null);
  let heading = $state<HTMLElement>();

  async function showExamples() {
    view = "examples";
    failure = null;
    try {
      list = await examples();
    } catch (e) {
      failure = String(e);
    }
  }

  // The heading takes focus when the page opens and when the view changes, so the next Tab is the first choice.
  $effect(() => {
    view;
    heading?.focus();
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onCancel();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="veil">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="start-title" data-testid="start-page">
    {#if view === "start"}
      <h2 id="start-title" tabindex="-1" bind:this={heading}>{t("app-start-title")}</h2>
      <p class="plain">{t("app-start-plain")}</p>
      <div class="choices">
        <button type="button" class="big first" data-testid="start-wizard" onclick={onWizard}>
          <span class="name">{t("app-start-wizard")}</span><span class="help">{t("app-start-wizard-help")}</span>
        </button>
        <button type="button" class="big" data-testid="start-example" onclick={showExamples}>
          <span class="name">{t("app-start-example")}</span><span class="help">{t("app-start-example-help")}</span>
        </button>
        <button type="button" class="big" data-testid="start-open" onclick={onOpen}>
          <span class="name">{t("app-start-open")}</span><span class="help">{t("app-start-open-help")}</span>
        </button>
      </div>
      <p class="skip"><button type="button" class="link" data-testid="start-skip" onclick={onSkip}>{t("app-start-skip")}</button></p>
    {:else}
      <h2 id="start-title" tabindex="-1" bind:this={heading}>{t("app-start-examples-title")}</h2>
      <div class="choices" data-testid="start-examples">
        {#if failure}
          <div class="msg" role="alert">{failure}</div>
        {:else if list === null}
          <p class="plain">{t("app-start-examples-loading")}</p>
        {:else}
          {#each list as ex (ex.id)}
            <button type="button" class="big" data-testid="start-example-{ex.id}" onclick={() => onExample(ex)}>
              <span class="name">{ex.title}</span><span class="help">{ex.text}</span>
            </button>
          {/each}
        {/if}
      </div>
      <p class="skip"><button type="button" class="link" data-testid="start-examples-back" onclick={() => (view = "start")}>{t("app-wizard-back")}</button></p>
    {/if}
    <footer>
      <button type="button" data-testid="start-cancel" onclick={onCancel}>{t("app-wizard-cancel")}</button>
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
    padding: 20px 26px; width: min(560px, calc(100vw - 32px)); box-sizing: border-box;
    box-shadow: 0 8px 30px color-mix(in srgb, var(--blueprint-deep, #1a3753) 40%, transparent);
  }
  h2 { font-size: 22px; margin: 0 0 4px; }
  h2:focus-visible { outline-offset: 4px; }
  .plain { margin: 0 0 var(--sp-3); font-size: var(--fs-14); color: var(--ink-2); }
  .choices { display: flex; flex-direction: column; gap: var(--sp-2); }
  button.big {
    display: flex; flex-direction: column; align-items: flex-start; gap: 2px; text-align: left; width: 100%;
    padding: var(--sp-3) var(--sp-4); background: var(--surface); border: var(--lw-thin) solid var(--rule-strong); border-radius: var(--radius);
  }
  button.big .name { font-size: var(--fs-16); font-weight: 600; }
  button.big .help { font-size: var(--fs-13); color: var(--ink-2); }
  button.big.first { background: var(--accent); border-color: var(--accent); color: var(--accent-ink); padding-block: var(--sp-4); }
  button.big.first .name { font-size: 20px; }
  button.big.first .help { color: var(--accent-ink); }
  button.big.first:hover:not(:disabled) { background: var(--accent); filter: brightness(1.12); }
  .skip { margin: var(--sp-3) 0 0; }
  .link { background: none; border: none; color: var(--accent); text-decoration: underline; padding: 0; cursor: pointer; font-size: var(--fs-14); }
  .link:hover:not(:disabled) { background: none; }
  .msg { color: var(--fail); font-size: var(--fs-13); }
  footer { display: flex; justify-content: flex-end; border-top: var(--lw-thin) solid var(--rule); margin-top: var(--sp-3); padding-top: var(--sp-3); }
</style>
