//! The damage case drawing (issue #761, spec 0012 D12 and D19): one sheet for the governing damage case of
//! each Rule Set and loading condition, not one per case. Each sheet has the plan of the hull with the
//! flooded Spaces hatched and, below it, the transverse section at the middle of the damage with the flooded
//! Spaces hatched and the damaged waterline.
//!
//! It reads the results only: the hull (as the GA views read it) and the `damage` block's `governing` list,
//! which holds each governing case's Flooded Volumes and its damaged waterplane. x is forward from the stern,
//! in plan y is to port; in the section y is to starboard and z is up from the base line, as the core's
//! axes.

use super::*;

/// The gap on paper between the plan and the section, in millimetres: room for the labels.
const SECTION_GAP_MM: f64 = 25.0;
/// The number of sides of the polygon a Type C tank's circle is hatched with.
const TANK_SIDES: usize = 36;

/// The message when an Option has no damage case drawing.
pub const NO_DAMAGE_DRAWING: &str =
    "a damage case drawing is drawn for an Option whose results hold damage cases (spec 0012)";

/// One Flooded Volume as the drawing shows it.
struct Flooded {
    name: String,
    x: (f64, f64),
    /// The section, (y, z) corners; for a tank the circle as a polygon.
    section: Vec<[f64; 2]>,
}

/// One governing case.
struct Governing {
    condition: String,
    case: String,
    ok: bool,
    floats: String,
    heel_deg: f64,
    trim_deg: f64,
    /// The damaged waterplane: heel, trim, height along its normal.
    waterplane: Option<(f64, f64, f64)>,
    flooded: Vec<Flooded>,
}

fn read_governing(results: &Value) -> Result<(String, Vec<Governing>), String> {
    let block = &results["damage"];
    let list = block["governing"]
        .as_array()
        .filter(|l| !l.is_empty())
        .ok_or_else(|| NO_DAMAGE_DRAWING.to_string())?;
    let set = block["rule_set"].as_str().unwrap_or("").to_string();
    let mut out = Vec::new();
    for g in list {
        let mut flooded = Vec::new();
        for f in g["flooded"].as_array().into_iter().flatten() {
            let x = (
                f["x_m"][0]
                    .as_f64()
                    .ok_or("a flooded Space has no x extent")?,
                f["x_m"][1]
                    .as_f64()
                    .ok_or("a flooded Space has no x extent")?,
            );
            let section: Vec<[f64; 2]> = if let Some(t) = f.get("tank") {
                let (y, z, d) = (
                    need(t, &["y_m"])?,
                    need(t, &["z_m"])?,
                    need(t, &["diameter_m"])?,
                );
                (0..TANK_SIDES)
                    .map(|k| {
                        let a = core::f64::consts::TAU * k as f64 / TANK_SIDES as f64;
                        let (s, c) = (libm::sin(a), libm::cos(a));
                        [y + 0.5 * d * c, z + 0.5 * d * s]
                    })
                    .collect()
            } else {
                f["section_m"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|p| Some([p[0].as_f64()?, p[1].as_f64()?]))
                    .collect()
            };
            flooded.push(Flooded {
                name: text_at(f, "name"),
                x,
                section,
            });
        }
        let wp = &g["waterplane"];
        out.push(Governing {
            condition: text_at(g, "condition"),
            case: text_at(g, "case"),
            ok: g["ok"].as_bool().unwrap_or(false),
            floats: text_at(g, "floats"),
            heel_deg: g["heel_deg"].as_f64().unwrap_or(0.0),
            trim_deg: g["trim_deg"].as_f64().unwrap_or(0.0),
            waterplane: match (
                wp["heel_deg"].as_f64(),
                wp["trim_deg"].as_f64(),
                wp["c_m"].as_f64(),
            ) {
                (Some(h), Some(t), Some(c)) if g["floats"] == "yes" => Some((h, t, c)),
                _ => None,
            },
            flooded,
        });
    }
    Ok((set, out))
}

/// The middle of the damage along the hull: the middle of the flooded Spaces' common length, or of their
/// whole extent when they share none.
fn damage_x(g: &Governing) -> f64 {
    let lo = g
        .flooded
        .iter()
        .map(|f| f.x.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let hi = g
        .flooded
        .iter()
        .map(|f| f.x.1)
        .fold(f64::INFINITY, f64::min);
    if lo < hi {
        return 0.5 * (lo + hi);
    }
    let lo = g
        .flooded
        .iter()
        .map(|f| f.x.0)
        .fold(f64::INFINITY, f64::min);
    let hi = g
        .flooded
        .iter()
        .map(|f| f.x.1)
        .fold(f64::NEG_INFINITY, f64::max);
    0.5 * (lo + hi)
}

fn damage_canvas(h: &Hull, g: &Governing, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let half = h.beam / 2.0;
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    // The plan: the hull, its rakes, the centre line; each flooded Space hatched over its extent across.
    c.polyline(
        "OUTLINE",
        &[[0.0, -half], [h.loa, -half], [h.loa, half], [0.0, half]],
        true,
    );
    if h.bow_rake > 0.0 {
        c.line(
            "OBJECT",
            [h.loa - h.bow_rake, -half],
            [h.loa - h.bow_rake, half],
            None,
        );
    }
    if h.stern_rake > 0.0 {
        c.line("OBJECT", [h.stern_rake, -half], [h.stern_rake, half], None);
    }
    c.centre_line([0.0, 0.0], [h.loa, 0.0]);
    for f in &g.flooded {
        let (y0, y1) = f
            .section
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| {
                (a.min(p[0]), b.max(p[0]))
            });
        if y0 >= y1 || f.x.0 >= f.x.1 {
            continue;
        }
        // plan y is to port: the core's starboard y is drawn below the centre line
        let (p0, p1) = (-y1, -y0);
        let rect = [[f.x.0, p0], [f.x.1, p0], [f.x.1, p1], [f.x.0, p1]];
        c.polyline("OBJECT", &rect, true);
        c.hatch("HATCH", &rect, "lines");
        labels.push((f.name.clone(), [0.5 * (f.x.0 + f.x.1), 0.5 * (p0 + p1)]));
    }
    let x = damage_x(g);
    c.line("CENTRE", [x, -half], [x, half], Some("dashed"));
    // The section at the middle of the damage, below the plan, centred on the damage's x.
    let dz = -half - SECTION_GAP_MM * c.k - h.depth;
    let at = |p: [f64; 2]| [x + p[0], dz + p[1]];
    let outline: Vec<[f64; 2]> = midship_outline(h).into_iter().map(at).collect();
    c.polyline("OUTLINE", &outline, false);
    c.line("OUTLINE", at([-half, h.depth]), at([half, h.depth]), None);
    c.centre_line(at([0.0, -0.5]), at([0.0, h.depth + 1.0]));
    for f in &g.flooded {
        if f.section.len() < 3 || !(f.x.0 <= x + 1e-9 && x <= f.x.1 + 1e-9) {
            continue;
        }
        let pts: Vec<[f64; 2]> = f.section.iter().map(|p| at(*p)).collect();
        c.polyline("OBJECT", &pts, true);
        c.hatch("HATCH", &pts, "lines");
    }
    // The damaged waterline: the waterplane at this x, across the section and a little beyond.
    if let Some((heel, trim, cc)) = g.waterplane {
        let deg = core::f64::consts::PI / 180.0;
        let (phi, theta) = (heel * deg, trim * deg);
        let (kx, ky, kz) = (
            -libm::sin(theta),
            -libm::sin(phi) * libm::cos(theta),
            libm::cos(phi) * libm::cos(theta),
        );
        let z_at = |y: f64| (cc - kx * x - ky * y) / kz;
        let reach = half * 1.15;
        c.line(
            "WATER",
            at([-reach, z_at(-reach)]),
            at([reach, z_at(reach)]),
            None,
        );
        labels.push((
            label!("draw-damaged-waterline", "DAMAGED WATERLINE"),
            at([reach, z_at(reach)]),
        ));
    }
    let column = c.max[0] + 6.0 * c.k;
    c.label_column(column, &mut labels);
    c
}

/// The damage case drawings of an Option's results: one sheet per governing case, each with its file-name
/// stem (`damage-1`, `damage-2`, ...). `option` names the Option in the title block. An Option with no damage
/// cases has none (an error with [`NO_DAMAGE_DRAWING`]).
pub fn damage_drawings_for(
    results: &Value,
    option: Option<&str>,
) -> Result<Vec<(String, Value)>, String> {
    let (set, governing) = read_governing(results)?;
    let mut h = read_hull(results)?;
    h.option_name = option.map(str::to_string);
    let title = label!("draw-view-damage-case", "DAMAGE CASE");
    let count = governing.len();
    let mut out = Vec::new();
    for (i, g) in governing.iter().enumerate() {
        let sheet = (i + 1) as u32;
        let mut notes = vec![label!(
            "draw-note-damage-case",
            "{view}: THE GOVERNING CASE OF THE CONDITION. FLOODED SPACES HATCHED. DIMENSIONS IN METRES.",
            view = title.clone()
        )];
        let what = plain_capitals(&format!("{}: {}", g.condition, g.case));
        if note_fits(&what) {
            notes.push(what);
        }
        notes.push(if g.floats == "yes" {
            label!(
                "draw-note-damage-position",
                "HEEL {heel} DEG, TRIM {trim} DEG. {verdict}",
                heel = fixed(g.heel_deg, 1),
                trim = fixed(g.trim_deg, 2),
                verdict = if g.ok {
                    label!("draw-damage-meets", "MEETS THE DAMAGE CRITERIA.")
                } else {
                    label!("draw-damage-fails", "DOES NOT MEET THE DAMAGE CRITERIA.")
                }
            )
        } else {
            label!(
                "draw-note-damage-no-float",
                "NO FLOATING POSITION AFTER DAMAGE: NO WATERLINE IS DRAWN."
            )
        });
        if let Some(sets) = rule_sets_note(core::slice::from_ref(&set)) {
            notes.push(sets);
        }
        notes.push(label!(
            "draw-note-scope",
            "FOR CONCEPT AND PRELIMINARY DESIGN ONLY, NOT FOR APPROVAL."
        ));
        let mut fields = title_fields(&h, &title, "DC", sheet, &plain_capitals(&h.vessel))?;
        fields["sheet_count"] = json!(count);
        let (doc, _) = place(notes, fields, RATIOS[0], &|n| damage_canvas(&h, g, n))?;
        out.push((format!("damage-{sheet}"), doc));
    }
    Ok(out)
}

/// As [`damage_drawings_for`], from a run outcome.
pub fn damage_drawings_named(
    outcome: &RunOutcome,
    option: Option<&str>,
) -> Result<Vec<(String, Value)>, String> {
    match outcome {
        RunOutcome::Results { results, .. } => damage_drawings_for(results, option),
        RunOutcome::NotAvailable(message) | RunOutcome::Failed(message) => Err(message.clone()),
    }
}
