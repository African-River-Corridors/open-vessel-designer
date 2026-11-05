<!-- unreleased -->
- Crew (issue #929): a river-only Option with no typed navigation hours now reads 24 h, RPN operating mode B (before, 14 h, mode A1); on a 140 x 29 m river-only deckhouse hull the manning minimum goes from 4 to 5 and the berths from 7 to 8. The RPN note no longer ties mode B to single rooms (RPN 18.02(4) is for modes A1 and A2 only). Type 14 h for the old default.

<!-- design-impact -->
### ticket/929-crew-mode-b (2026-11-05)

Between `main 960b606` and `this branch d6636191`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas river-only deckhouse 140x29 | `checks[9].message` | 4 tier(s) of 488.6 m2 hold the 57.7 m2 of accommodation floor and, for a fixed wheelhouse, lift the eye over the cargo; over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | 4 tier(s) of 488.6 m2 hold the 63.4 m2 of accommodation floor and, for a fixed wheelhouse, lift the eye over the cargo; over 4 tiers is a tall house over a short machinery space (ESTIMATE limit) | changed |
