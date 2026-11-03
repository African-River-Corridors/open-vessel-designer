//! The cargo detail sheet of a Type C tank (issue #844): an elevation and a cross-section of one
//! cargo tank, on one sheet at one scale. The elevation shows the shell and the heads, the dome, the
//! insulation and the support block; the cross-section shows the same at mid-length. The sheet is
//! for an Option whose Cargo Containment is Type C tanks; a box hold and the liquid tanks have none.
//!
//! Every figure on it is read from the results (`drawing.tank`, `drawing`, `arrangement`). The one
//! number the sheet calculates is the volume at the filling limit, per tank: `arrangement.per_tank_m3`
//! times `arrangement.fill_ratio`, and the notes say it is derived. The results hold no saddle
//! geometry (issue #752): the support block has the run's height, a width that is a drawing
//! proportion, and the label ESTIMATE.

use super::*;

/// The support block in elevation is this share of the cylinder length wide, and in section this
/// share of the tank diameter. Drawing proportions: the results have no saddle width (issue #752).
const SUPPORT_WIDTH_OF_LENGTH: f64 = 0.2;
const SUPPORT_WIDTH_OF_DIAMETER: f64 = 0.3;
/// The width of the TYPE box of the title block, less its padding (iso7200: 25 mm, 1 mm each side).
const TYPE_BOX_MM: f64 = 23.0;
/// The pitch of the stack of leader labels above the elevation, in millimetres: the text height and a
/// gap of 1.5 mm.
const TOP_STEP_MM: f64 = 5.0;
/// The paper distance of the length dimension below the support block, in millimetres.
const LENGTH_DIM_MM: f64 = 8.0;

/// The tank as the sheet draws it, read from the results.
struct Tank {
    /// Outside the insulation (`tank_outer_diameter_m`) and of the steel (`tank_steel_outer_diameter_m`).
    od: f64,
    steel_od: f64,
    /// The cylinder between the head joints, and the length overall with the heads.
    cylinder_length: f64,
    overall_length: f64,
    insulation_mm: f64,
    cylinder_wall_mm: f64,
    head_wall_mm: Option<f64>,
    /// The dome: height and diameter.
    dome: Option<(f64, f64)>,
    support: f64,
    head_ratio: f64,
    design_pressure: f64,
    internal_pressure: f64,
    temperature: Option<f64>,
    material: String,
    per_tank_m3: f64,
    fill_ratio: f64,
}

fn read_tank(h: &Hull, results: &Value) -> Result<Tank, String> {
    let tank = |key: &str| need(results, &["drawing", "tank", key]);
    let Containment::Tanks {
        od,
        support_height,
        head_ratio,
        ..
    } = &h.containment
    else {
        return Err("the Option has no Type C tanks".to_string());
    };
    let dome = match (
        num_at(results, &["drawing", "tank", "dome_height_m"]),
        num_at(results, &["drawing", "tank", "dome_diameter_m"]),
    ) {
        (Some(rise), Some(d)) if rise > 0.0 && d > 0.0 => Some((rise, d)),
        _ => None,
    };
    Ok(Tank {
        od: *od,
        steel_od: tank("tank_steel_outer_diameter_m")?,
        cylinder_length: tank("cylinder_length_m")?,
        overall_length: tank("overall_tank_length_m")?,
        insulation_mm: tank("insulation_thickness_mm")?,
        cylinder_wall_mm: tank("cylinder_wall_thickness_mm")?,
        head_wall_mm: num_at(results, &["drawing", "tank", "head_wall_thickness_mm"]),
        dome,
        support: *support_height,
        head_ratio: *head_ratio,
        design_pressure: tank("design_pressure_mpa")?,
        internal_pressure: tank("internal_pressure_mpa")?,
        temperature: num_at(results, &["drawing", "tank", "design_temperature_c"]),
        material: plain_capitals(
            results["drawing"]["tank"]["material"]
                .as_str()
                .unwrap_or_default(),
        ),
        per_tank_m3: need(results, &["arrangement", "per_tank_m3"])?,
        fill_ratio: need(results, &["arrangement", "fill_ratio"])?,
    })
}

impl Tank {
    /// The volume at the filling limit, per tank: derived, not a result of the run.
    fn filled_m3(&self) -> f64 {
        self.per_tank_m3 * self.fill_ratio
    }

    /// The dome as a quadratic curve on the outer surface of the tank whose axis is at height `y0`, as
    /// the midship section draws a dome. Gives the height of its apex.
    fn dome_curve(&self, c: &mut Canvas, x: f64, y0: f64) -> Option<f64> {
        let (rise, d) = self.dome?;
        let r = self.od / 2.0;
        let half = (d / 2.0).min(0.95 * r);
        let base = (r * r - half * half).sqrt();
        let sag = r + rise - base;
        c.dome("OBJECT", x, y0 + base, half, sag);
        Some(y0 + r + rise)
    }
}

fn mm(x: f64) -> String {
    format!("{} mm", fixed(x, 1))
}

/// The labels the two views share, as (text, target) pairs are built by the view; the texts are here.
fn text_cylinder_wall(t: &Tank) -> String {
    label!(
        "draw-cd-cylinder-wall",
        "CYLINDER WALL {thickness}",
        thickness = mm(t.cylinder_wall_mm)
    )
}

fn text_steel_diameter(t: &Tank) -> String {
    label!(
        "draw-cd-steel-diameter",
        "STEEL OUTER DIAMETER {diameter}",
        diameter = metres(t.steel_od)
    )
}

fn text_outer_diameter(t: &Tank) -> String {
    label!(
        "draw-cd-outer-diameter",
        "OUTER DIAMETER OVER INSULATION {diameter}",
        diameter = metres(t.od)
    )
}

fn text_insulation(t: &Tank) -> String {
    label!(
        "draw-cd-insulation",
        "INSULATION {thickness}",
        thickness = format!("{} mm", fixed(t.insulation_mm, 0))
    )
}

fn text_dome(rise: f64) -> String {
    label!(
        "draw-cd-dome",
        "DOME HEIGHT {height}",
        height = metres(rise)
    )
}

fn text_support(t: &Tank) -> String {
    label!(
        "draw-cd-support",
        "SUPPORT BLOCK {height} (ESTIMATE)",
        height = metres(t.support)
    )
}

/// The elevation of the tank: axis at height 0, the left head joint at x = 0.
fn elevation(t: &Tank, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let (r_o, r_s, l) = (t.od / 2.0, t.steel_od / 2.0, t.cylinder_length);
    let xm = l / 2.0;
    c.tank_outline("OUTLINE", (0.0, l, 0.0, r_o), t.head_ratio);
    c.tank_outline("OBJECT", (0.0, l, 0.0, r_s), t.head_ratio);
    // the support block under the shell
    // the width and the place of the block are a drawing proportion, not saddle data
    let half = SUPPORT_WIDTH_OF_LENGTH * l / 2.0;
    c.polyline(
        "OBJECT",
        &[
            [xm - half, -r_o],
            [xm - half, -r_o - t.support],
            [xm + half, -r_o - t.support],
            [xm + half, -r_o],
        ],
        false,
    );
    let apex = t.dome_curve(&mut c, xm, 0.0);
    let tip = t.head_ratio * r_o;
    let overshoot = 4.0 * c.k;
    c.centre_line([-tip - overshoot, 0.0], [l + tip + overshoot, 0.0]);
    // the length overall, under the support block
    let drop = (r_o + t.support) / c.k + LENGTH_DIM_MM;
    let length_dimension = c.try_dim_horizontal([-tip, 0.0], [l + tip, 0.0], -drop);
    // The wall, the insulation and the dome are leaders that rise from the top of the tank to a
    // stack of labels above it: a leader to a label level with its target would run along the outline.
    // The leftmost target has the highest label, so no two leaders cross.
    let mut top: Vec<(String, [f64; 2])> = vec![
        (text_insulation(t), [0.62 * l, (r_o + r_s) / 2.0]),
        (text_cylinder_wall(t), [0.76 * l, r_s]),
    ];
    if let Some(w) = t.head_wall_mm {
        let a = core::f64::consts::FRAC_PI_4;
        top.push((
            label!(
                "draw-cd-head-wall",
                "HEAD WALL {thickness}",
                thickness = mm(w)
            ),
            [l + t.head_ratio * r_s * libm::sin(a), r_s * libm::cos(a)],
        ));
    }
    if let (Some(apex), Some((rise, _))) = (apex, t.dome) {
        top.insert(0, (text_dome(rise), [xm, apex]));
    }
    let mut bottom: Vec<(String, [f64; 2])> = vec![
        (text_steel_diameter(t), [0.62 * l, -r_s]),
        (text_outer_diameter(t), [0.8 * l, -r_o]),
        (text_support(t), [xm + half, -r_o - t.support / 2.0]),
    ];
    if !length_dimension {
        bottom.push((
            label!(
                "draw-cd-length",
                "OVERALL LENGTH {length}",
                length = metres(t.overall_length)
            ),
            [l + tip, 0.0],
        ));
    }
    let column = c.max[0] + 6.0 * c.k;
    let first = r_o + 2.0 * c.k;
    let count = top.len();
    for (i, (text, target)) in top.iter().enumerate() {
        let level = first + (count - 1 - i) as f64 * TOP_STEP_MM * c.k;
        c.leader(text, *target, [column, level]);
    }
    c.label_column(column, &mut bottom);
    c
}

/// The cross-section at mid-length. The two views of the sheet have windows of the model that must
/// not overlap (clause 15.4.9), so the section stands this many millimetres of paper below the
/// elevation: far more than a window is high.
const SECTION_DROP_MM: f64 = 400.0;

fn section(t: &Tank, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let oy = -SECTION_DROP_MM * c.k;
    let at = |x: f64, y: f64| [x, oy + y];
    let (r_o, r_s) = (t.od / 2.0, t.steel_od / 2.0);
    c.circle("OUTLINE", at(0.0, 0.0), r_o);
    c.circle("OBJECT", at(0.0, 0.0), r_s);
    let half = SUPPORT_WIDTH_OF_DIAMETER * t.od / 2.0;
    c.polyline(
        "OBJECT",
        &[
            at(-half, -r_o),
            at(-half, -r_o - t.support),
            at(half, -r_o - t.support),
            at(half, -r_o),
        ],
        false,
    );
    let apex = t.dome_curve(&mut c, 0.0, oy);
    let overshoot = 4.0 * c.k;
    c.centre_line(
        at(0.0, -r_o - t.support - overshoot),
        at(0.0, r_o + overshoot),
    );
    c.centre_line(at(-r_o - overshoot, 0.0), at(r_o + overshoot, 0.0));
    // the outer diameter, to the left of the tank, if it has room for its arrowheads
    c.try_dim_vertical(at(0.0, -r_o), at(0.0, r_o), r_o / c.k + DIM_OFFSET_MM);
    let on = |r: f64, deg: f64| {
        at(
            r * libm::cos(deg.to_radians()),
            r * libm::sin(deg.to_radians()),
        )
    };
    let mut labels: Vec<(String, [f64; 2])> = vec![
        (text_cylinder_wall(t), on(r_s, 20.0)),
        (text_steel_diameter(t), on(r_s, -25.0)),
        (text_outer_diameter(t), on(r_o, -50.0)),
        (text_insulation(t), on((r_o + r_s) / 2.0, 45.0)),
        (text_support(t), at(half, -r_o - t.support / 2.0)),
    ];
    if let (Some(apex), Some((rise, _))) = (apex, t.dome) {
        labels.push((text_dome(rise), [0.0, apex]));
    }
    let column = c.max[0] + 6.0 * c.k;
    c.label_column(column, &mut labels);
    c
}

/// The cargo detail sheet of an Option with Type C tanks.
pub(super) fn cargo_detail_sheet(h: &Hull, results: &Value) -> Result<Value, String> {
    let t = read_tank(h, results)?;
    let title = label!("draw-view-cargo-detail", "CARGO DETAIL");
    let mut notes = vec![label!(
        "draw-note-cargo-detail",
        "{view}: ELEVATION AND CROSS-SECTION OF ONE CARGO TANK, AT ONE SCALE. DIMENSIONS IN METRES.",
        view = title.clone()
    )];
    notes.push(label!(
        "draw-note-tank-pressure",
        "DESIGN VAPOUR PRESSURE {design} MPA. INTERNAL PRESSURE WITH THE LIQUID HEAD {internal} MPA.",
        design = fixed(t.design_pressure, 3),
        internal = fixed(t.internal_pressure, 3)
    ));
    let condition = match t.temperature {
        Some(temp) => label!(
            "draw-note-tank-temperature",
            "DESIGN TEMPERATURE {temperature} DEG C. MATERIAL {material}.",
            temperature = fixed(temp, 0),
            material = t.material.clone()
        ),
        None => label!(
            "draw-note-tank-material",
            "MATERIAL {material}.",
            material = t.material.clone()
        ),
    };
    let volume = label!(
        "draw-note-tank-volume",
        "VOLUME AT FILLING LIMIT, PER TANK: {volume} M3 (DERIVED: {per_tank} X {fill}).",
        volume = fixed(t.filled_m3(), 1),
        per_tank = fixed(t.per_tank_m3, 1),
        fill = fixed(t.fill_ratio, 2)
    );
    // one line when the two fit together: the strip is the height the views lose
    let joined = format!("{condition} {volume}");
    if note_fits(&joined) {
        notes.push(joined);
    } else {
        notes.push(condition);
        notes.push(volume);
    }
    notes.push(label!(
        "draw-note-support-estimate",
        "SUPPORT BLOCK: THE HEIGHT IS THE RUN'S, ITS WIDTH AND POSITION ARE A DRAWING PROPORTION. ESTIMATE, NOT SADDLE DATA."
    ));
    if let Some(sets) = rule_sets_note(&h.rule_sets) {
        notes.push(sets);
    }
    notes.push(label!(
        "draw-note-scope",
        "FOR CONCEPT AND PRELIMINARY DESIGN ONLY, NOT FOR APPROVAL."
    ));
    notes.extend(failed_notes(&h.failed_checks));
    let mut fields = title_fields(h, &title, "CD", 1, &plain_capitals(&h.vessel))?;
    fields["sheet_count"] = json!(1);
    // The TYPE box of the title block reads the sheet's own title. Its cell is 25 mm wide at text
    // height 3.5 and a text is never cut (clause 14.6): when the whole title does not fit, the
    // last word of it, which names the kind of sheet, stands in the box.
    let kind = if fits_mm(&title, 3.5, TYPE_BOX_MM) {
        title.clone()
    } else {
        title.rsplit(' ').next().unwrap_or(&title).to_string()
    };
    fields["document_type"] = json!(kind);
    place_pair(notes, fields, RATIOS[0], &|n| {
        [elevation(&t, n), section(&t, n)]
    })
    .map(|(d, _)| d)
}
