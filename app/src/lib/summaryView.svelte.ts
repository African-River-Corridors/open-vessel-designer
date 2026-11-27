// Display state of the Summary view that lasts the session: whether its scope sentence is open. Display state only.
export const summaryView = $state<{ scopeOpen: boolean }>({ scopeOpen: false });
