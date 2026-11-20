<!-- unreleased -->
- The web tests remove a browser's profile only once no process uses it any more, so a Chrome still writing to it for a moment after its session ends no longer fails a test that passed (`ENOTEMPTY`). A removal that still fails is reported; the run's own clean-up removes the folder, and the run still fails if any of its processes is alive.
