<!-- unreleased -->
- Gas fuel system allowances (issue #775, spec 0011 D8): a gas-fuelled stern block, when its mass is in the lightship, gains gas valve units (0.4 t per gas engine), a gas heater (0.2 kg per kWe) and a gas compressor (0.6 kg per kWe) as items of their own, and a gas combustion unit (2 t) when the new `gas_combustion_unit` input is true. No source was read for any of them, so each is marked "estimate" with its basis in `stern_block_mass.gas_fuel_system`; the calibration ticket replaces them. Reliquefaction is not modelled. The gas fuel system cost line now prices the installed gas genset rating. Lightship, centres and the dependent results move on gas-fuelled Options; the new input defaults to false, and a file saved before keeps false.

<!-- design-impact -->
### ticket/775-gas-fuel-allowances (2026-11-02)

Between `main 9141340` and `this branch a80ec453`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 5.98 m (fluid), max GZ 2.54 m at 29° | GM 5.97 m (fluid), max GZ 2.54 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.80 m (fluid), max GZ 3.48 m at 27° | GM 12.78 m (fluid), max GZ 3.48 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 12.91 m (fluid), max GZ 3.47 m at 27° | GM 12.89 m (fluid), max GZ 3.47 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.gas_combustion_unit` | absent | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[0].basis` | absent | estimate: 0.4 t per gas engine, the reserve set included; no source read | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[0].label` | absent | gas valve units | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[0].mass_t` | absent | 0.8 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[0].status` | absent | estimate | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[1].basis` | absent | estimate: 0.2 kg per kWe of the gas gensets; no source read | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[1].label` | absent | gas heater | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[1].mass_t` | absent | 0.48 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[1].status` | absent | estimate | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[2].basis` | absent | estimate: 0.6 kg per kWe of the gas gensets; no source read | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[2].label` | absent | gas compressor | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[2].mass_t` | absent | 1.44 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.items[2].status` | absent | estimate | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.reliquefaction` | absent | not modelled | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.served_kwe` | absent | 2400 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.gas_fuel_system.total_t` | absent | 2.72 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[10].label` | small items | frequency converters | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[11].label` | piping and cabling | electric motors | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[12].label` | engine-room steel | bow thruster | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[13].label` | engine seats | small items | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[14].kcg_m` | absent | 0.586666666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[14].label` | absent | piping and cabling | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[14].lcg_m` | absent | 8.916 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[14].mass_t` | absent | 6.14961129407 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[15].kcg_m` | absent | 3.14333333333 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[15].label` | absent | engine-room steel | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[15].lcg_m` | absent | 11.616 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[15].mass_t` | absent | 40.058728736 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[16].kcg_m` | absent | 0.586666666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[16].label` | absent | engine seats | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[16].lcg_m` | absent | 11.616 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[16].mass_t` | absent | 3.2 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[5].label` | propulsors | gas valve units | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[6].label` | switchboards | gas heater | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[7].label` | frequency converters | gas compressor | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[8].label` | electric motors | propulsors | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stern_block_mass.items[9].label` | bow thruster | switchboards | changed |
| ... | 86876 more changed quantities | | | |
