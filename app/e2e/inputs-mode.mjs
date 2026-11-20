// Issue #1084: the Input panel opens on Essentials, which hides the inputs that use the Defaults Profile. A test
// that needs one of those inputs switches to All inputs with this one helper. `browser` is a WebdriverIO browser.
export async function showAllInputs(browser) {
  await browser.execute(() => {
    const all = document.querySelector("[data-testid=mode-all]");
    if (all && all.getAttribute("aria-pressed") !== "true") all.click();
  });
}
