<script lang="ts">
  // One small picture of the vessel on the Summary: the Drawings view's own sheet, with only its hull geometry
  // shown, framed on the hull (vessel.ts). Nothing is drawn here. It is a button: a click, Enter or Space opens
  // the drawing in the Drawings view. Its box has a fixed height, so a redraw (after damage stability) never
  // moves what is below it.
  import { frameOf, HULL_GEOMETRY } from "./vessel";

  let { svg, label, hint, testid, onOpen }: {
    /** The sheet, as the Drawings view shows it. */
    svg: string;
    /** What the picture shows, for a screen reader. */
    label: string;
    /** What a click does. */
    hint: string;
    testid: string;
    onOpen: () => void;
  } = $props();

  let holder: HTMLDivElement;

  // Frame the sheet on the hull each time the drawing changes: measure the hull's elements, set the view box.
  $effect(() => {
    void svg;
    const sheet = holder?.querySelector("svg");
    if (!sheet) return;
    const boxes = [...sheet.querySelectorAll<SVGGraphicsElement>(HULL_GEOMETRY)].map((e) => e.getBBox());
    const frame = frameOf(boxes, 0.03);
    if (frame) sheet.setAttribute("viewBox", `${frame.x} ${frame.y} ${frame.width} ${frame.height}`);
    sheet.removeAttribute("width");
    sheet.removeAttribute("height");
    sheet.setAttribute("preserveAspectRatio", "xMidYMid meet");
    sheet.setAttribute("aria-hidden", "true");
  });
</script>

<button type="button" class="picture" data-testid={testid} aria-label={label} title={hint} onclick={onOpen}>
  <div class="holder" bind:this={holder}>{@html svg}</div>
</button>

<style>
  /* The sheet is white in both themes, as in the Drawings view: it is paper. */
  .picture { display: block; width: 100%; height: 100%; padding: var(--sp-1); box-sizing: border-box; background: var(--sheet); border: var(--lw-thin) solid var(--rule-strong); border-radius: 0; cursor: pointer; }
  .picture:hover:not(:disabled) { background: var(--sheet); border-color: var(--accent); }
  .holder { width: 100%; height: 100%; }
  .holder :global(svg) { display: block; width: 100%; height: 100%; }
  /* Only the hull: the title block, notes, dimensions, leaders and scale bar stay in the Drawings view. */
  .holder :global(svg > :not(style):not([data-origin="viewport.main"])) { display: none; }
</style>
