/** Where Tab or Shift+Tab sends focus in a dialog (WAI-ARIA dialog pattern). No DOM in this function. */
export function wrapTarget<T>(items: readonly T[], active: T | null, backward: boolean): T | null {
  if (items.length === 0) return null;
  const at = active === null ? -1 : items.indexOf(active);
  // Focus is not on a listed element (the page, or an element a step change removed): enter at the matching edge.
  if (at === -1) return backward ? items[items.length - 1] : items[0];
  if (!backward && at === items.length - 1) return items[0];
  if (backward && at === 0) return items[items.length - 1];
  return null; // inside the list: the browser moves focus
}

const CANDIDATES = 'a[href], button, input, select, textarea, summary, [tabindex]';

/** The elements in `root` that take focus from Tab now: enabled, shown, not inert, one radio to a group. */
export function tabbable(root: HTMLElement): HTMLElement[] {
  const found = [...root.querySelectorAll<HTMLElement>(CANDIDATES)].filter((el) => {
    if ((el as HTMLButtonElement).disabled || el.tabIndex < 0) return false;
    if (el.closest("[hidden], [inert]") || el.getClientRects().length === 0) return false;
    return getComputedStyle(el).visibility !== "hidden";
  });
  return found.filter((el) => {
    if (!(el instanceof HTMLInputElement) || el.type !== "radio" || !el.name) return true;
    const group = found.filter((o) => o instanceof HTMLInputElement && o.type === "radio" && o.name === el.name && o.form === el.form);
    return el === (group.find((o) => (o as HTMLInputElement).checked) ?? group[0]);
  });
}

/** Svelte action: Tab and Shift+Tab wrap at the first and last element of the dialog. */
export function trapFocus(node: HTMLElement) {
  function onKey(e: KeyboardEvent) {
    if (e.key !== "Tab" || e.altKey || e.ctrlKey || e.metaKey) return;
    const active = document.activeElement instanceof HTMLElement && node.contains(document.activeElement) ? document.activeElement : null;
    const to = wrapTarget(tabbable(node), active, e.shiftKey);
    if (!to) return;
    e.preventDefault();
    to.focus();
  }
  // On the window, so a Tab still wraps when focus fell to the page after a step removed the focused element.
  window.addEventListener("keydown", onKey);
  return { destroy: () => window.removeEventListener("keydown", onKey) };
}
