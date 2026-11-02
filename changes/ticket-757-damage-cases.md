<!-- unreleased -->
- Damage case generator (issue #757, spec 0012): a new `damage_cases` module holds one generator for every rule with box extents. The ADN and MARPOL generators moved there unchanged; the IGC generator adds the IGC 2.3.1 extents and the IGC 2.6.1 standard of damage for a type 2G ship, read from the new `igc-2g` 3.0.0 tables. No result moves.
