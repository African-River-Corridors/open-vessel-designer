<!-- unreleased -->
- The web harness's own tests never leave a helper process behind: the stand-in writer exits when its test file's process is gone and after 30 s at most, every case stops whatever of it is still running, and the cases' folders live under the run's root, which the run script removes.
