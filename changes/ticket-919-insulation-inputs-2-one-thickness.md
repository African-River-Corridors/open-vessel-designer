<!-- unreleased -->
- One insulation thickness (issue #919): the general-arrangement adapter reads the tank design's thickness where a capacity result carries none, not a stand-in of 350 mm. The 350 mm constant is now `LEGACY_SIMULATE_INSULATION_MM`, the original engine's value for a hand-built tank design and for `simulate` params without the key; a Design never reads it. No result moves.
