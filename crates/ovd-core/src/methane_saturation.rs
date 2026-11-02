//! Saturated methane: pressure, densities, internal energies and enthalpies against temperature,
//! and the gauge and absolute pressure basis (issue #867, spec 0016 section 4).
//!
//! Every 1 C from -170 C to -100 C, from the NIST Chemistry WebBook, Thermophysical Properties of
//! Fluid Systems, saturation properties of methane (default reference state), as kept in
//! `catalogue/methane_saturation/nist_methane.tsv` (a test keeps the table equal to that file, which
//! holds the columns of the NIST file the code reads, as printed). Between two rows the value is the
//! straight line between them. Outside the table every function returns [`OutOfTable`]: a value is
//! never clamped.
//!
//! The rules give pressures as gauge; the table is absolute. They differ by one standard atmosphere,
//! 0.101325 MPa.

use libm::{floor, round};

/// One standard atmosphere (MPa): a gauge pressure plus this is the absolute pressure.
pub const ATMOSPHERE_MPA: f64 = 0.101325;
/// The lowest temperature of the table (C); a row every 1 C above it.
pub const TABLE_LOW_C: f64 = -170.0;
/// The highest temperature of the table (C).
pub const TABLE_HIGH_C: f64 = -100.0;

/// A request outside the table (temperature or pressure).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutOfTable;

/// A pressure that carries its basis, so that a gauge value and an absolute value convert both ways
/// without rounding: the value given is the value returned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pressure {
    gauge_mpa: f64,
    absolute_mpa: f64,
}

impl Pressure {
    /// From a gauge pressure (MPa), the basis of the rules.
    pub fn from_gauge_mpa(gauge_mpa: f64) -> Self {
        Pressure {
            gauge_mpa,
            absolute_mpa: gauge_mpa + ATMOSPHERE_MPA,
        }
    }
    /// From an absolute pressure (MPa), the basis of the table.
    pub fn from_absolute_mpa(absolute_mpa: f64) -> Self {
        Pressure {
            gauge_mpa: absolute_mpa - ATMOSPHERE_MPA,
            absolute_mpa,
        }
    }
    pub fn gauge_mpa(&self) -> f64 {
        self.gauge_mpa
    }
    pub fn absolute_mpa(&self) -> f64 {
        self.absolute_mpa
    }
}

/// One row: (pressure bar absolute, liquid density kg/m3, vapour density kg/m3, liquid internal
/// energy kJ/kg, vapour internal energy kJ/kg, liquid enthalpy kJ/kg, vapour enthalpy kJ/kg).
type Row = (f64, f64, f64, f64, f64, f64, f64);

/// The table, from -170 C.
#[rustfmt::skip]
pub static METHANE: [Row; 71] = [
    (0.47229, 434.51, 0.90242, -29.594, 443.65, -29.485, 495.99), // -170 C
    (0.52016, 433.11, 0.98585, -26.169, 445.03, -26.049, 497.79), // -169 C
    (0.57175, 431.70, 1.0751, -22.739, 446.40, -22.607, 499.58), // -168 C
    (0.62727, 430.29, 1.1703, -19.303, 447.75, -19.158, 501.35), // -167 C
    (0.68691, 428.87, 1.2719, -15.862, 449.10, -15.702, 503.11), // -166 C
    (0.75087, 427.44, 1.3800, -12.414, 450.44, -12.239, 504.85), // -165 C
    (0.81936, 426.00, 1.4949, -8.9608, 451.76, -8.7685, 506.57), // -164 C
    (0.89260, 424.56, 1.6170, -5.5011, 453.08, -5.2909, 508.28), // -163 C
    (0.97079, 423.11, 1.7465, -2.0351, 454.38, -1.8056, 509.96), // -162 C
    (1.0542, 421.65, 1.8836, 1.4374, 455.67, 1.6874, 511.63), // -161 C
    (1.1429, 420.18, 2.0288, 4.9165, 456.94, 5.1885, 513.28), // -160 C
    (1.2373, 418.71, 2.1822, 8.4023, 458.21, 8.6978, 514.91), // -159 C
    (1.3376, 417.22, 2.3442, 11.895, 459.46, 12.216, 516.52), // -158 C
    (1.4439, 415.73, 2.5151, 15.395, 460.70, 15.742, 518.11), // -157 C
    (1.5565, 414.23, 2.6952, 18.902, 461.92, 19.278, 519.67), // -156 C
    (1.6757, 412.72, 2.8848, 22.416, 463.13, 22.822, 521.22), // -155 C
    (1.8017, 411.20, 3.0843, 25.938, 464.32, 26.377, 522.74), // -154 C
    (1.9347, 409.67, 3.2940, 29.468, 465.51, 29.941, 524.24), // -153 C
    (2.0750, 408.13, 3.5142, 33.006, 466.67, 33.514, 525.72), // -152 C
    (2.2228, 406.58, 3.7453, 36.552, 467.82, 37.099, 527.17), // -151 C
    (2.3784, 405.02, 3.9877, 40.106, 468.95, 40.694, 528.60), // -150 C
    (2.5420, 403.45, 4.2416, 43.669, 470.07, 44.299, 530.00), // -149 C
    (2.7139, 401.87, 4.5076, 47.241, 471.17, 47.916, 531.37), // -148 C
    (2.8943, 400.28, 4.7859, 50.822, 472.25, 51.545, 532.73), // -147 C
    (3.0835, 398.68, 5.0770, 54.412, 473.31, 55.185, 534.05), // -146 C
    (3.2817, 397.06, 5.3813, 58.011, 474.36, 58.838, 535.34), // -145 C
    (3.4893, 395.43, 5.6992, 61.621, 475.39, 62.503, 536.61), // -144 C
    (3.7064, 393.79, 6.0311, 65.240, 476.39, 66.181, 537.85), // -143 C
    (3.9333, 392.14, 6.3775, 68.870, 477.38, 69.873, 539.06), // -142 C
    (4.1703, 390.47, 6.7389, 72.511, 478.35, 73.579, 540.24), // -141 C
    (4.4177, 388.79, 7.1156, 76.163, 479.30, 77.299, 541.38), // -140 C
    (4.6757, 387.09, 7.5083, 79.826, 480.22, 81.033, 542.50), // -139 C
    (4.9446, 385.38, 7.9173, 83.500, 481.13, 84.783, 543.58), // -138 C
    (5.2246, 383.66, 8.3433, 87.187, 482.01, 88.549, 544.63), // -137 C
    (5.5160, 381.92, 8.7868, 90.886, 482.87, 92.331, 545.65), // -136 C
    (5.8192, 380.16, 9.2483, 94.598, 483.70, 96.129, 546.63), // -135 C
    (6.1343, 378.39, 9.7285, 98.324, 484.51, 99.945, 547.57), // -134 C
    (6.4617, 376.60, 10.228, 102.06, 485.30, 103.78, 548.48), // -133 C
    (6.8016, 374.79, 10.747, 105.82, 486.06, 107.63, 549.35), // -132 C
    (7.1543, 372.96, 11.287, 109.58, 486.79, 111.50, 550.18), // -131 C
    (7.5201, 371.11, 11.848, 113.37, 487.50, 115.39, 550.97), // -130 C
    (7.8992, 369.25, 12.431, 117.17, 488.18, 119.31, 551.72), // -129 C
    (8.2920, 367.36, 13.037, 120.98, 488.83, 123.24, 552.43), // -128 C
    (8.6987, 365.45, 13.667, 124.81, 489.45, 127.19, 553.09), // -127 C
    (9.1196, 363.53, 14.321, 128.66, 490.03, 131.17, 553.71), // -126 C
    (9.5550, 361.57, 15.000, 132.53, 490.59, 135.17, 554.29), // -125 C
    (10.005, 359.60, 15.706, 136.42, 491.12, 139.20, 554.82), // -124 C
    (10.470, 357.60, 16.439, 140.33, 491.61, 143.25, 555.30), // -123 C
    (10.951, 355.57, 17.201, 144.25, 492.06, 147.33, 555.73), // -122 C
    (11.447, 353.52, 17.992, 148.20, 492.48, 151.44, 556.11), // -121 C
    (11.959, 351.44, 18.813, 152.17, 492.86, 155.58, 556.43), // -120 C
    (12.488, 349.32, 19.667, 156.17, 493.21, 159.74, 556.70), // -119 C
    (13.033, 347.18, 20.554, 160.19, 493.51, 163.94, 556.91), // -118 C
    (13.595, 345.01, 21.476, 164.23, 493.77, 168.17, 557.07), // -117 C
    (14.174, 342.80, 22.435, 168.30, 493.98, 172.44, 557.16), // -116 C
    (14.770, 340.56, 23.431, 172.40, 494.15, 176.74, 557.19), // -115 C
    (15.384, 338.28, 24.468, 176.53, 494.28, 181.08, 557.15), // -114 C
    (16.017, 335.96, 25.546, 180.69, 494.35, 185.46, 557.05), // -113 C
    (16.668, 333.61, 26.669, 184.88, 494.37, 189.88, 556.87), // -112 C
    (17.337, 331.20, 27.838, 189.11, 494.34, 194.34, 556.62), // -111 C
    (18.026, 328.76, 29.056, 193.37, 494.25, 198.85, 556.28), // -110 C
    (18.734, 326.26, 30.326, 197.67, 494.09, 203.41, 555.87), // -109 C
    (19.462, 323.71, 31.650, 202.01, 493.88, 208.03, 555.37), // -108 C
    (20.210, 321.11, 33.033, 206.40, 493.60, 212.69, 554.77), // -107 C
    (20.978, 318.45, 34.478, 210.83, 493.24, 217.42, 554.09), // -106 C
    (21.767, 315.72, 35.989, 215.31, 492.81, 222.21, 553.29), // -105 C
    (22.578, 312.93, 37.570, 219.85, 492.30, 227.06, 552.39), // -104 C
    (23.410, 310.07, 39.228, 224.44, 491.70, 231.99, 551.38), // -103 C
    (24.264, 307.13, 40.967, 229.09, 491.01, 236.99, 550.24), // -102 C
    (25.140, 304.10, 42.794, 233.81, 490.22, 242.07, 548.97), // -101 C
    (26.040, 300.98, 44.717, 238.60, 489.32, 247.25, 547.56), // -100 C
];

/// The saturation properties of one column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Property {
    PressureBar,
    LiquidDensity,
    VapourDensity,
    LiquidInternalEnergy,
    VapourInternalEnergy,
    LiquidEnthalpy,
    VapourEnthalpy,
}

fn pick(r: &Row, p: Property) -> f64 {
    match p {
        Property::PressureBar => r.0,
        Property::LiquidDensity => r.1,
        Property::VapourDensity => r.2,
        Property::LiquidInternalEnergy => r.3,
        Property::VapourInternalEnergy => r.4,
        Property::LiquidEnthalpy => r.5,
        Property::VapourEnthalpy => r.6,
    }
}

/// A property at `t_c`: the table value at a row, the straight line between two rows.
pub fn property(p: Property, t_c: f64) -> Result<f64, OutOfTable> {
    let x = t_c - TABLE_LOW_C;
    if x.is_nan() || x < 0.0 || x > (METHANE.len() - 1) as f64 {
        return Err(OutOfTable);
    }
    let i = floor(x) as usize;
    if i + 1 >= METHANE.len() {
        return Ok(pick(&METHANE[i], p));
    }
    let f = x - i as f64;
    let (a, b) = (pick(&METHANE[i], p), pick(&METHANE[i + 1], p));
    Ok(if f == 0.0 { a } else { a + f * (b - a) })
}

/// The saturation pressure at `t_c` (MPa absolute).
pub fn saturation_pressure_mpa(t_c: f64) -> Result<f64, OutOfTable> {
    Ok(property(Property::PressureBar, t_c)? / 10.0)
}
/// The saturated liquid density at `t_c` (kg/m3).
pub fn liquid_density_kg_m3(t_c: f64) -> Result<f64, OutOfTable> {
    property(Property::LiquidDensity, t_c)
}
/// The saturated vapour density at `t_c` (kg/m3).
pub fn vapour_density_kg_m3(t_c: f64) -> Result<f64, OutOfTable> {
    property(Property::VapourDensity, t_c)
}
/// The saturated liquid specific internal energy at `t_c` (kJ/kg).
pub fn liquid_internal_energy_kj_per_kg(t_c: f64) -> Result<f64, OutOfTable> {
    property(Property::LiquidInternalEnergy, t_c)
}
/// The saturated vapour specific internal energy at `t_c` (kJ/kg).
pub fn vapour_internal_energy_kj_per_kg(t_c: f64) -> Result<f64, OutOfTable> {
    property(Property::VapourInternalEnergy, t_c)
}
/// The saturated liquid specific enthalpy at `t_c` (kJ/kg).
pub fn liquid_enthalpy_kj_per_kg(t_c: f64) -> Result<f64, OutOfTable> {
    property(Property::LiquidEnthalpy, t_c)
}
/// The saturated vapour specific enthalpy at `t_c` (kJ/kg).
pub fn vapour_enthalpy_kj_per_kg(t_c: f64) -> Result<f64, OutOfTable> {
    property(Property::VapourEnthalpy, t_c)
}

/// The latent heat at `t_c` (kJ/kg): vapour enthalpy less liquid enthalpy, taken at each row to the
/// 0.01 kJ/kg the boil-off figures have always used, then the straight line between rows.
pub fn latent_heat_kj_per_kg(t_c: f64) -> Result<f64, OutOfTable> {
    let x = t_c - TABLE_LOW_C;
    if x.is_nan() || x < 0.0 || x > (METHANE.len() - 1) as f64 {
        return Err(OutOfTable);
    }
    let row = |i: usize| round((METHANE[i].6 - METHANE[i].5) * 100.0) / 100.0;
    let i = floor(x) as usize;
    if i + 1 >= METHANE.len() {
        return Ok(row(i));
    }
    let f = x - i as f64;
    let a = row(i);
    Ok(if f == 0.0 {
        a
    } else {
        a + f * (row(i + 1) - a)
    })
}

/// The temperature (C) at which the saturation pressure is `p_mpa` absolute: the inverse of
/// [`saturation_pressure_mpa`] on the same straight lines.
pub fn saturation_temperature_c(p_mpa: f64) -> Result<f64, OutOfTable> {
    let bar = p_mpa * 10.0;
    if bar.is_nan() || bar < METHANE[0].0 || bar > METHANE[METHANE.len() - 1].0 {
        return Err(OutOfTable);
    }
    for i in 1..METHANE.len() {
        let (lo, hi) = (METHANE[i - 1].0, METHANE[i].0);
        if bar <= hi {
            return Ok(TABLE_LOW_C + (i - 1) as f64 + (bar - lo) / (hi - lo));
        }
    }
    Err(OutOfTable)
}
