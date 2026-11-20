<!-- unreleased -->
- The web harness's own test file stops and removes only its own folder at its end; before, it killed every process that named the run's shared temp root, which took down the browsers of the test files running beside it.
