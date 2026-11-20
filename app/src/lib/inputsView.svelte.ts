// What the Input panel shows, as state the panel and the rest of the app share (issue #1084): the mode
// (Essentials or All inputs, kept per user, like the theme) and the text in the filter box (not kept).
import { loadMode, saveMode, type InputsMode } from "./inputFilter";

export const inputsView = $state<{ mode: InputsMode; query: string }>({ mode: loadMode(), query: "" });

/** Switch between Essentials and All inputs, and keep the choice. A Design made by the wizard opens on Essentials: call `setInputsMode("essentials")`. */
export function setInputsMode(mode: InputsMode) {
  inputsView.mode = mode;
  saveMode(mode);
}

export function setInputsFilter(query: string) {
  inputsView.query = query;
}
