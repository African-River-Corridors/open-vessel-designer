import { tick } from "svelte";
import { inputsView, setInputsFilter, setInputsMode } from "./inputsView.svelte";

const fieldOf = (name: string) => document.querySelector<HTMLElement>(`[data-input="${CSS.escape(name)}"]`);

/**
 * Scroll to an input in the left panel, open its group if closed, and focus its field. An input that
 * Essentials or the filter hides (issue #1084) is reached by switching to All inputs and clearing the filter.
 */
export async function focusInput(name: string) {
  let field = fieldOf(name);
  if (!field) {
    if (inputsView.mode !== "all") setInputsMode("all");
    if (inputsView.query !== "") setInputsFilter("");
    await tick();
    field = fieldOf(name);
  }
  if (!field) return;
  const group = field.closest("details");
  if (group && !group.open) group.open = true;
  field.scrollIntoView({ block: "center" });
  field.querySelector<HTMLElement>("input, select")?.focus();
}
