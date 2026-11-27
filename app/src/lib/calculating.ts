/**
 * The sign "Calculating…" (issue #1082, spec 0017 D13): on from the moment the wait passes `afterMs` until
 * every evaluation sent has come back. Runs can overlap, so a count of runs in flight decides, not a flag:
 * the first run to finish must not hide the sign while another still runs.
 *
 * Issue #1096: on the web an evaluation runs on the page's own thread and blocks it, so the timer cannot fire
 * and the page cannot paint until the run is over. A run the caller expects to be slow (`slow`) turns the sign
 * on at once and waits for `paint` (the page drawing the sign) before the work starts.
 */
export class Calculating {
  private inFlight = 0;
  private shown = false;
  /** The page has drawn a frame since the sign came on. */
  private drawn = false;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private readonly afterMs: number;
  private readonly onChange: (on: boolean) => void;
  private readonly paint: () => Promise<unknown>;

  constructor(afterMs: number, onChange: (on: boolean) => void, paint: () => Promise<unknown> = nextPaint) {
    this.afterMs = afterMs;
    this.onChange = onChange;
    this.paint = paint;
  }

  /** Runs in flight now. */
  get running(): number {
    return this.inFlight;
  }

  /** The page changed in a way the designer must see before the next slow run: wait for one more frame then. */
  redraw() {
    this.drawn = false;
  }

  private show() {
    this.shown = true;
    this.onChange(true);
  }

  /** Run `work`; the sign is off again when it ends, whether it returns or throws. */
  async during<T>(work: () => Promise<T>, slow = false): Promise<T> {
    this.inFlight += 1;
    if (this.inFlight === 1) this.timer = setTimeout(() => this.show(), this.afterMs);
    try {
      if (slow) {
        if (!this.shown) {
          clearTimeout(this.timer);
          this.show();
        }
        // On in the page is not yet on the screen (the timer may have just turned it on): wait for a frame.
        if (!this.drawn) {
          await this.paint();
          this.drawn = true;
        }
      }
      return await work();
    } finally {
      this.inFlight -= 1;
      if (this.inFlight === 0) {
        clearTimeout(this.timer);
        this.shown = false;
        this.drawn = false;
        this.onChange(false);
      }
    }
  }
}

/**
 * Resolve once the page has drawn what was just changed: a task queued from an animation frame runs after
 * that frame is painted. A visible page always waits for that frame, however slow; a hidden page draws no
 * frames, so there a short timer ends the wait.
 */
export function nextPaint(): Promise<void> {
  return new Promise((resolve) => {
    const hidden = typeof document === "undefined" || document.visibilityState === "hidden";
    if (typeof requestAnimationFrame !== "function" || hidden) {
      setTimeout(resolve, 0);
      return;
    }
    requestAnimationFrame(() => setTimeout(resolve, 0));
  });
}

/**
 * The latest run asked for wins. `ask()` gives a check that stays true until a newer run is asked for: a run that
 * is still waiting (for the page to draw the sign) when a newer one is asked for skips its work, and the newer one
 * gives the answer. After Create the start-up evaluation could still be waiting for a frame; both then evaluated
 * the new Design, one after the other, and on the web each costs seconds.
 */
export class Latest {
  private asked = 0;

  ask(): () => boolean {
    const mine = ++this.asked;
    return () => mine === this.asked;
  }
}
