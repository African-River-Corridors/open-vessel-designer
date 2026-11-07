<!-- unreleased -->
- Internal (issue #935): the gas damage case loop, the IGC damage data and the Permeability Tables moved into `ovd_core::gas_damage` as typed values, so the arrangement search can call them. `ovd-format` keeps the worker threads, the progress hook and the JSON block. No design change.
