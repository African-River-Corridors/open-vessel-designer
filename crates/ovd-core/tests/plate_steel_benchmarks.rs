//! Issue #745: the plate model beside the published midship sections of
//! `calibration/plate_steel_benchmarks.yaml` (issue #751). Run with `--nocapture` for the table the
//! pull request records. A benchmark, never a target: nothing is tuned to it (spec 0013, D16).
//!
//! Label: sourced - calibration/plate_steel_benchmarks.yaml, the published values quoted with their
//! table or page there (rank 1: J. Mar. Sci. Eng. 2021, 9, 99, Tables 1, 5 and 7, Section 2.1; rank 3:
//! the owner's spec sheet, page 1). The model's values are printed, not pinned.
//!
//! What each run takes that the source does not give is an Estimate: Cb 0.9 (a box barge), no
//! double bottom and no double side (both are single-hull deck barges), the cargo zone the middle 84 %
//! of the length, no deck openings. Rank 3 publishes no spacing, so it takes the model's 700 mm and
//! 3.66 m. Both were classed to other rules (Lloyd's Register; an older ABS Ocean edition), so the ABS
//! Steel Barges minimums of the model need not meet them.

use ovd_core::hull_panels::{HullStructureSpec, function_type as ft, region};
use ovd_core::hull_plates::ItemKind;
use ovd_core::vessel_model::StructureBlock;

struct Case {
    id: &'static str,
    loa: f64,
    rule_length: f64,
    beam: f64,
    depth: f64,
    holds: usize,
    abreast: usize,
    long_mm: f64,
    frame_m: f64,
}

fn spec(c: &Case) -> HullStructureSpec {
    HullStructureSpec {
        loa_m: c.loa,
        rule_length_m: c.rule_length,
        beam_m: c.beam,
        depth_m: c.depth,
        block_coefficient: 0.9,
        cargo_zone_aft_m: 0.08 * c.loa,
        cargo_zone_fwd_m: 0.92 * c.loa,
        double_bottom_m: 0.0,
        wing_width_m: 0.0,
        wing_top_m: c.depth,
        deck_opening_width_m: 0.0,
        wing_deck_opening_width_m: 0.0,
        holds_along: c.holds,
        tanks_abreast: c.abreast,
        steel_density: 7.85,
        structure: StructureBlock {
            longitudinal_spacing_mm: c.long_mm,
            frame_spacing_m: c.frame_m,
            ..StructureBlock::default()
        },
    }
}

#[test]
fn the_benchmark_table() {
    // (case, published bottom, side, deck plate mm; bottom longitudinal area cm2; deck section
    // modulus cm2-m). None: not published.
    let cases = [
        (
            Case {
                id: "lr-ocean-deck-barge-122x30-mx (rank 1)",
                loa: 122.45,
                // Table 1: loadline length (L) 119.95 m.
                rule_length: 119.95,
                beam: 30.5,
                depth: 7.6,
                // Section 2.1: three cargo tanks; three longitudinal bulkheads, so four abreast.
                holds: 3,
                abreast: 4,
                // Section 2.1: longitudinals at 0.7 m, transverse frames at 2.5 m.
                long_mm: 700.0,
                frame_m: 2.5,
            },
            // Table 7: bottom 22 mm; Section 1: original deck 22 mm; Table 7: the angle bar
            // 282.6 x 17.4 + 90 x 17.4 = 64.8 cm2; Table 5: ZD 5.338 m3 (with the corroded 14 mm deck).
            (Some(22.0), None, Some(22.0), Some(64.8), Some(53_380.0)),
        ),
        (
            Case {
                id: "abs-ocean-deck-barge-260x72 (rank 3)",
                // Derived in the yaml: 79.25 x 21.95 x 4.88 m.
                loa: 79.25,
                rule_length: 0.96 * 79.25,
                beam: 21.95,
                depth: 4.88,
                // Page 1: six transverse bulkheads, 15 midbody compartments.
                holds: 5,
                abreast: 3,
                long_mm: 700.0,
                frame_m: 3.66,
            },
            // Page 1: bottom 1/2", side 1/2", deck 9/16" (derived 12.7, 12.7, 14.3 mm); bottom
            // longitudinals 5" x 3-1/2" x 5/16" angles (not converted to an area here).
            (Some(12.7), Some(12.7), Some(14.3), None, None),
        ),
    ];
    println!(
        "| case | bottom mm (model / published) | side mm | deck mm | bottom longitudinal cm2 | deck SM cm2-m | ordered steel t |"
    );
    println!("|---|---|---|---|---|---|---:|");
    let show = |m: f64, p: Option<f64>| match p {
        Some(p) => format!("{m:.1} / {p:.1} ({:+.0} %)", (m / p - 1.0) * 100.0),
        None => format!("{m:.1} / -"),
    };
    for (c, (bottom, side, deck, long_cm2, sm)) in &cases {
        let s = spec(c);
        let (panels, plates) = ovd_core::hull_plates::gas_hull_steel(&s, None);
        let mid = |member: &str| {
            plates
                .items
                .iter()
                .find(|i| {
                    i.kind == ItemKind::Plate && i.member == member && i.region == region::MIDSHIP
                })
                .map_or(0.0, |i| i.ordered_mm)
        };
        let long = panels
            .members
            .iter()
            .find(|m| m.member == ft::BOTTOM_SHELL && m.region == region::MIDSHIP)
            .and_then(|m| m.stiffener.as_ref())
            .map_or(0.0, |s| s.area_cm2());
        let g = panels.hull_girder;
        println!(
            "| {} | {} | {} | {} | {} | {} | {:.0} |",
            c.id,
            show(mid(ft::BOTTOM_SHELL), *bottom),
            show(mid(ft::SIDE_SHELL), *side),
            show(mid(ft::STRENGTH_DECK), *deck),
            show(long, *long_cm2),
            show(g.sm_deck_cm2m, *sm),
            plates.ordered_t,
        );
        assert!(
            plates.ordered_t.is_finite() && plates.ordered_t > 0.0,
            "{}",
            c.id
        );
        assert!(
            mid(ft::BOTTOM_SHELL) > 0.0 && mid(ft::STRENGTH_DECK) > 0.0,
            "{}",
            c.id
        );
    }
}
