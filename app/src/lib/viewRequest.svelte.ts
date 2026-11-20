// Which view of the Views area a caller asks for (issue #1081). Any part of the app can call `showView`; the Views
// panel reads the request and selects the view. The ids are the tab ids of MainPanel.svelte: "summary", "side",
// "plan", "midship", "convoy", "model3d", "stern", "stability", "ballast", "damage", "speed-power", "rules",
// "boq", "capital". Display state only.
export const viewRequest = $state<{ id: string | null; count: number }>({ id: null, count: 0 });

/** Ask for a view: the same id asked twice still counts as a new request. */
export function showView(id: string) {
  viewRequest.id = id;
  viewRequest.count += 1;
}

/** Issue #1097: the tab id of the view the Views panel shows now; the app opens or closes the Outputs panel by it. */
export const viewShown = $state<{ id: string }>({ id: "summary" });
