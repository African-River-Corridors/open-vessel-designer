// Issue #1097: the Summary view takes the whole main area, so the Outputs panel is closed while it shows. A test
// that reads the Outputs panel right after a Design opens shows it with this one helper. `browser` is a
// WebdriverIO browser.
export async function showOutputsPanel(browser) {
  await browser.execute(() => {
    const button = document.querySelector("[data-testid=toggle-outputs-panel]");
    if (button && button.getAttribute("aria-pressed") !== "true") button.click();
  });
  await browser.waitUntil(() => browser.execute(() => !document.querySelector("section.bottom")?.hidden), {
    timeout: 5000,
    timeoutMsg: "the Outputs panel to show",
  });
}
