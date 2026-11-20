/** Finds one message by its id in one language: its text, or null where the language has none. */
export type Finder = (id: string) => string | null;

/**
 * Which text the app shows for an engine message (issue #1082). The app's own wording comes first, in the
 * Display Language and then in English: the engine's English may point to the command line or the Python
 * API, which the app never does. Without app wording, English shows the engine's text as it is, and another
 * language shows its translation of the engine message, or the engine's English where it has none.
 */
export function wordingOf(engine: string, id: string, display: Finder, source: Finder, displayIsSource: boolean): string {
  const plain = `app-plain-${id}`;
  const own = display(plain) ?? source(plain);
  if (own !== null) return own;
  if (displayIsSource) return engine;
  return display(id) ?? engine;
}
