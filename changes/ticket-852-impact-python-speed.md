<!-- unreleased -->
- CI (issue #852): the fixture-hash check runs in `gate-lint`, beside the Rule Set gate that builds the same `ovd`; `gate-impact` builds no Python extension and runs the design-impact cases on a release `ovd` (the same answers to the bit); the edge-sweep test runs its Options in parallel processes; the pytest shards upload their test times.
