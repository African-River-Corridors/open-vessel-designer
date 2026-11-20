// Which side panels are shown. The choice is display state kept in local storage, per user. Storage
// can be empty or not available (a private window, blocked site data): then both panels are shown.
export interface PanelChoice {
  input: boolean;
  outputs: boolean;
}

export const PANELS_KEY = "ovd.panels";
const SHOWN: PanelChoice = { input: true, outputs: true };

export function loadPanels(): PanelChoice {
  try {
    const saved = JSON.parse(localStorage.getItem(PANELS_KEY) ?? "null");
    return {
      input: saved?.input !== false,
      outputs: saved?.outputs !== false,
    };
  } catch {
    return { ...SHOWN };
  }
}

export function savePanels(choice: PanelChoice) {
  try {
    localStorage.setItem(PANELS_KEY, JSON.stringify(choice));
  } catch {
    // Not remembered; the panels still work.
  }
}

/**
 * Issue #1097: the Summary takes the whole main area, so a designer's first answer is one page. On the Summary
 * the Outputs panel shows only when the designer opened it there this session (`onSummary`); on any other view
 * the saved choice (`outputs`) decides, as before.
 */
export interface OutputsState {
  /** The tab id of the view shown. */
  view: string;
  /** The saved choice, for every view but the Summary. */
  outputs: boolean;
  /** The designer opened the Outputs panel on the Summary this session. */
  onSummary: boolean;
}

export function outputsShown(s: OutputsState): boolean {
  return s.view === "summary" ? s.onSummary : s.outputs;
}

/** The Outputs panel button: it changes the choice of the view shown, and nothing else. */
export function toggleOutputs(s: OutputsState): OutputsState {
  return s.view === "summary" ? { ...s, onSummary: !s.onSummary } : { ...s, outputs: !s.outputs };
}
