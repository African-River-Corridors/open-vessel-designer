//! Seam: gross tonnage by ITC 1969 (issue #777, spec 0011 D15), against the Convention's own table
//! of K1 and against the published GT of two built flat-top deck barges.
//!
//! Label: sourced - ITC 1969 Annex I Regulation 3 and Appendix 2, UN Treaty Series vol. 1291, No. I-21264 (1982), pp. 15 and 21; barge "LM 3004" particulars sheet (gob.mx, 2021); Mubarak Marine "Mubarak CB-03" particulars sheet (2022)
//!
//! - Appendix 2 tabulates K1 "at intermediate values of V ... by linear interpolation" to four
//!   decimals; each row read here must equal 0.2 + 0.02 log10 V to that rounding.
//! - Neither barge sheet gives V, so each test brackets it from published values and checks that the
//!   published GT lies between GT of the two brackets. Upper bracket: L x B x D, the moulded box that
//!   holds the whole hull of a flat-top barge with no deckhouse (neither sheet lists one; a
//!   sideboard or a ramp is open, not an enclosed space under Regulation 2(4)). Lower bracket: the
//!   displaced volume at the published draught (displacement / 1.025 t/m3 sea water), which lies
//!   inside V. This tests the formula and its use of V, not OVD's hull geometry.

use ovd_core::tonnage::{GrossTonnage, gross_tonnage, k1};

/// Appendix 2 rows (V m3, K1), read from the UN Treaty Series copy, p. 21.
const APPENDIX_2: &[(f64, f64)] = &[
    (10.0, 0.2200),
    (20.0, 0.2260),
    (100.0, 0.2400),
    (1_000.0, 0.2600),
    (5_000.0, 0.2740),
    (10_000.0, 0.2800),
    (45_000.0, 0.2931),
    (100_000.0, 0.3000),
    (330_000.0, 0.3104),
    (1_000_000.0, 0.3200),
];

#[test]
fn k1_matches_the_conventions_appendix_2_table() {
    for &(v, table) in APPENDIX_2 {
        let k = k1(v);
        assert!(
            (k - table).abs() <= 0.00005,
            "V {v} m3: K1 {k} against the table's {table}"
        );
    }
}

#[test]
fn gt_is_k1_times_v() {
    // Regulation 3 at V = 10,000 m3: K1 = 0.2 + 0.02 x 4 = 0.28, GT = 2,800.
    assert!((gross_tonnage(10_000.0) - 2_800.0).abs() < 1e-9);
    // And at V = 1,000 m3: K1 = 0.26, GT = 260.
    assert!((gross_tonnage(1_000.0) - 260.0).abs() < 1e-9);
}

/// The published GT lies between GT of the lower and the upper bracket of V.
fn bracketed(name: &str, published_gt: f64, v_low: f64, v_high: f64) {
    let (lo, hi) = (gross_tonnage(v_low), gross_tonnage(v_high));
    assert!(
        lo <= published_gt && published_gt <= hi,
        "{name}: published GT {published_gt} not within [{lo:.1}, {hi:.1}] from V in [{v_low:.1}, {v_high:.1}] m3"
    );
}

#[test]
fn barge_lm_3004_published_gt_lies_within_its_volume_brackets() {
    // Barge "LM 3004", built 2007, Panama flag, IBS class: LOA 87.86 m, moulded breadth 24.4 m,
    // moulded depth 5.5 m, draft 4.3 m, GRT/NRT 3172/951, lightship 1,450 MT, deadweight 7,388 MT
    // (particulars sheet published on gob.mx, file 730399, created 2021-11-09).
    let box_v = 87.86 * 24.4 * 5.5;
    let displaced_v = (1_450.0 + 7_388.0) / 1.025;
    bracketed("LM 3004", 3_172.0, displaced_v, box_v);
}

#[test]
fn barge_mubarak_cb_03_published_gt_lies_within_its_volume_brackets() {
    // Mubarak CB-03, flat top cargo barge, built 2006, UAE flag, Bureau Veritas: LOA 36.6 m, breadth
    // 12.1 m, depth 2.4 m, max draft 1.8 m, GRT 272 t, displacement 828.2 t (Mubarak Marine
    // particulars sheet CB-03, 2022).
    let box_v = 36.6 * 12.1 * 2.4;
    let displaced_v = 828.2 / 1.025;
    bracketed("Mubarak CB-03", 272.0, displaced_v, box_v);
}

#[test]
fn a_typed_gt_overrides_the_computed_one() {
    let v = ovd_core::tonnage::EnclosedVolume {
        hull_m3: 10_000.0,
        ..Default::default()
    };
    let computed = GrossTonnage::of(v, None);
    assert_eq!(computed.source(), "computed");
    assert!((computed.value() - 2_800.0).abs() < 1e-9);
    let typed = GrossTonnage::of(v, Some(2_950.0));
    assert_eq!(typed.source(), "typed");
    assert_eq!(typed.value(), 2_950.0);
    // The computed value is still shown beside the typed one.
    assert!((typed.computed - 2_800.0).abs() < 1e-9);
}
