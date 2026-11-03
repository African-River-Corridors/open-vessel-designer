//! The accommodation rooms of issue #780 (spec 0011 D5, section 2), sized from ES-TRIN 2025/1
//! Chapter 15 (the text read in the rule sources: `es-trin/ES_TRIN_2025_signed_en.txt`, pages 103-107).
//!
//! Issue #892 adds the sea-reaching rooms, sized to the greater of MLC 2006 (as amended, 2022
//! consolidated text, in the rule sources `mlc-2006/`) Standard A3.1 and ES-TRIN.
//!
//! Label: golden values, each worked by hand from the paragraph quoted beside it, not read back from
//! the code.

use ovd_core::rooms::*;

fn basis(berths: u32) -> RoomsBasis {
    RoomsBasis {
        berths,
        reaches_sea: false,
        gt: 0,
        officers: 0,
        seafarers: 0,
        mode_b: false,
    }
}

fn sea(berths: u32, officers: u32, gt: u32, seafarers: u32) -> RoomsBasis {
    RoomsBasis {
        berths,
        reaches_sea: true,
        gt,
        officers,
        seafarers,
        mode_b: false,
    }
}

fn row<'a>(r: &'a Rooms, id: &str) -> &'a Room {
    r.rows.iter().find(|x| x.id == id).expect(id)
}

#[test]
fn one_single_cabin_is_the_larger_of_the_volume_air_and_berth_floors() {
    // 15.02(9): 7 m3 at 15.02(6) headroom 2.00 m = 3.50 m2. 15.02(10): 5 m3 of air = 2.50 m2.
    // 15.07(1): a berth of 2.00 x 0.90 m = 1.80 m2. The largest is 3.50 m2.
    let r = size_rooms(&basis(1));
    let c = row(&r, "cabin");
    assert_eq!(c.count, 1);
    assert!((c.area_each_m2 - 3.5).abs() < 1e-12);
}

#[test]
fn six_berths_by_hand() {
    // cabins 6 x 3.50 = 21.00 (15.02(9), (10), (6))
    // communal room: 2 m2 x 6 = 12.00, over the 8 m2 floor (15.02(8))
    // toilets: 6 / 6 = 1 compartment x 1.00 m2 (15.03(1)(a), (3))
    // corridor: 0.60 m wide (15.02(11)(a) clear door width) x (6 cabins / 2 per bay = 3 bays x
    // 1.75 m cabin front (3.50 m2 / 2.00 m berth length)) = 0.60 x 5.25 = 3.15
    // sourced/derived total 21.00 + 12.00 + 1.00 + 3.15 = 37.15
    // estimates: washbasins ceil(6/4) = 2 x 0.5 = 1.00; shower ceil(6/6) = 1 x 0.81 = 0.81;
    // galley 5.00; work clothes 2.0 + 0.25 x 6 = 3.50; stairway 1.60 -> 11.91
    // total 37.15 + 11.91 = 49.06
    let r = size_rooms(&basis(6));
    assert!((row(&r, "cabin").area_m2 - 21.0).abs() < 1e-9);
    assert!((row(&r, "communal").area_m2 - 12.0).abs() < 1e-9);
    assert_eq!(row(&r, "toilet").count, 1);
    assert!((row(&r, "toilet").area_m2 - 1.0).abs() < 1e-9);
    assert!((row(&r, "corridor").area_m2 - 3.15).abs() < 1e-9);
    assert!((r.total_m2 - 49.06).abs() < 1e-9);
}

#[test]
fn a_small_crew_takes_the_8_m2_communal_floor() {
    // 1 berth: 2 m2 is under the 8 m2 total of 15.02(8): 8.00. Cabin 3.50, toilet 1.00,
    // corridor 0.60 x (1 bay x 1.75) = 1.05: 13.55. Estimates: basin 0.5, shower 0.81, galley 5.0,
    // work clothes 2.25, stairway 1.6 = 10.16. Total 23.71.
    let r = size_rooms(&basis(1));
    assert!((row(&r, "communal").area_m2 - 8.0).abs() < 1e-9);
    assert!((row(&r, "corridor").area_m2 - 1.05).abs() < 1e-9);
    assert!((r.total_m2 - 23.71).abs() < 1e-9);
}

#[test]
fn thirteen_berths_and_the_fixture_counts() {
    // cabins 13 x 3.5 = 45.5; communal 26.0; toilets ceil(13/6) = 3 -> 3.00;
    // corridor 0.6 x (7 bays x 1.75) = 7.35; 81.85 so far.
    // Basins ceil(13/4) = 4 x 0.5 = 2.00 and showers ceil(13/6) = 3 x 0.81 = 2.43 (15.03(1)(b), (c));
    // galley 5.0; work clothes 2.0 + 0.25 x 13 = 5.25; stairway 1.6. Total 81.85 + 16.28 = 98.13.
    let r = size_rooms(&basis(13));
    assert!((r.total_m2 - 98.13).abs() < 1e-9);
    assert_eq!(row(&r, "washbasin").count, 4);
    assert_eq!(row(&r, "shower").count, 3);
}

#[test]
fn every_row_has_a_clause_and_a_basis() {
    for n in [1, 4, 13] {
        for row in &size_rooms(&basis(n)).rows {
            assert!(row.clause.starts_with("ES-TRIN 2025/1 "), "{}", row.id);
            assert!(
                ["sourced", "derived", "estimate"].contains(&row.basis),
                "{}",
                row.id
            );
            assert!((row.area_m2 - row.count as f64 * row.area_each_m2).abs() < 1e-9);
        }
    }
}

fn estimate(n: u32, id: &str, area_each: f64, count: u32) {
    let r = size_rooms(&basis(n));
    let x = row(&r, id);
    assert_eq!(x.basis, "estimate", "{id}");
    assert_eq!(x.count, count, "{id}");
    assert!((x.area_each_m2 - area_each).abs() < 1e-9, "{id}");
    assert!(x.note.contains("ESTIMATE"), "{id}");
}

#[test]
fn the_washbasin_estimate_is_half_a_square_metre_each() {
    estimate(6, "washbasin", 0.5, 2);
}

#[test]
fn the_shower_estimate_is_a_90_cm_cubicle() {
    estimate(6, "shower", 0.9 * 0.9, 1);
}

#[test]
fn the_galley_estimate_is_5_m2() {
    // 2.4 x (0.6 + 1.0) = 3.84 + 1.2 stores = 5.04, rounded to 5.0.
    estimate(6, "galley", 5.0, 1);
}

#[test]
fn the_work_clothes_estimate_grows_a_quarter_metre_a_berth() {
    estimate(6, "work_clothes", 3.5, 1);
    estimate(13, "work_clothes", 5.25, 1);
}

#[test]
fn the_stairway_estimate_is_one_stairway_of_1_6_m2() {
    // 0.60 x (13 x 0.15 + 0.60) = 0.60 x 2.55 = 1.53, rounded to 1.6.
    estimate(6, "stairs", 1.6, 1);
}

#[test]
fn every_mode_gets_one_cabin_per_berth_on_the_same_rule() {
    // One cabin per crew member is OVD's own cautious rule in every mode; the mode changes no row.
    let b = RoomsBasis {
        berths: 4,
        reaches_sea: false,
        gt: 0,
        officers: 0,
        seafarers: 0,
        mode_b: true,
    };
    let (rb, ra) = (size_rooms(&b), size_rooms(&basis(4)));
    assert_eq!(row(&rb, "cabin").count, 4);
    assert_eq!(rb, ra);
    assert_eq!(row(&rb, "cabin").basis, "derived");
    assert!(row(&rb, "cabin").note.contains("OVD's own cautious rule"));
}

#[test]
fn a_river_hull_ignores_the_tonnage_and_the_officers() {
    // River-only: ES-TRIN alone, whatever the tonnage or officers say.
    let b = RoomsBasis {
        berths: 6,
        reaches_sea: false,
        gt: 12_000,
        officers: 4,
        seafarers: 20,
        mode_b: false,
    };
    let r = size_rooms(&b);
    assert!((r.total_m2 - 49.06).abs() < 1e-9);
    assert!(!r.mlc_applied);
    assert!((r.headroom_m - 2.00).abs() < 1e-12);
    assert!(r.rows.iter().all(|x| x.governing == "es-trin"));
    assert!(r.rows.iter().all(|x| x.id != "hospital"));
}

#[test]
fn the_gt_bands_have_the_edges_of_a3_1_9_f() {
    // A3.1.9(f): (i) under 3,000; (ii) 3,000 or over but under 10,000; (iii) 10,000 or over.
    assert_eq!(gt_band(2_999), 0);
    assert_eq!(gt_band(3_000), 1);
    assert_eq!(gt_band(9_999), 1);
    assert_eq!(gt_band(10_000), 2);
    // Not known: the largest band, the cautious one.
    assert_eq!(gt_band(0), 2);
}

#[test]
fn a_rating_cabin_follows_a3_1_9_f_by_gross_tonnage() {
    // A3.1.9(f)(i) 4.5, (ii) 5.5, (iii) 7 m2: each over the ES-TRIN 3.50 m2 (15.02(9): 7 m3 / 2.00 m).
    for (gt, want) in [(2_999, 4.5), (3_000, 5.5), (9_999, 5.5), (10_000, 7.0)] {
        let r = size_rooms(&sea(1, 0, gt, 1));
        let c = row(&r, "cabin");
        assert!((c.area_each_m2 - want).abs() < 1e-12, "{gt}");
        assert_eq!(c.governing, "mlc");
        assert!(c.clause.contains("MLC 2006 as amended"), "{gt}");
        assert!(c.clause.contains("Standard A3.1 9(f)"), "{gt}");
        assert!(c.clause.contains("ES-TRIN 2025/1"), "{gt}");
    }
}

#[test]
fn eight_berths_at_5000_gt_by_hand() {
    // 8 berths, 4 of them officers, 5,000 GT (band (ii)), 6 seafarers (no hospital).
    // A3.1.9(m): three officers (master, chief engineer, chief navigating officer) have a day room, so
    // their sleeping rooms take the A3.1.9(f)(ii) 5.5 m2: 3 x 5.5 = 16.5; day rooms 3 x 6.0 = 18.0
    // (ESTIMATE). The fourth officer has none, so A3.1.9(k)(ii) 8.5 m2. Ratings 4 x 5.5 = 22.0.
    // Cabins 22.0 + 16.5 + 8.5 = 47.0 m2.
    // Communal: 2 x 8 = 16.0 (15.02(8)). Toilets ceil(8/6) = 2 x 1.0 = 2.0 (A3.1.11(c), 15.03(3)).
    // Washbasins: one in each of 8 sleeping rooms (A3.1.11(d)) x 0.5 = 4.0 (ES-TRIN: 2).
    // Showers ceil(8/6) = 2 x 0.81 = 1.62. Corridor: mean cabin 47.0 / 8 = 5.875 m2, front over the
    // 2.00 m berth = 2.9375 m; 4 bays; 0.60 x 4 x 2.9375 = 7.05. Galley 5.0; work clothes
    // 2.0 + 0.25 x 8 = 4.0; stairway 1.6.
    // Total 47.0 + 18.0 + 16.0 + 2.0 + 4.0 + 1.62 + 7.05 + 5.0 + 4.0 + 1.6 = 106.27.
    let r = size_rooms(&sea(8, 4, 5_000, 6));
    assert!(r.mlc_applied);
    assert!((r.headroom_m - 2.03).abs() < 1e-12);
    let cabin = row(&r, "cabin");
    assert_eq!(cabin.count, 4);
    assert!((cabin.area_m2 - 22.0).abs() < 1e-9);
    let senior = row(&r, "cabin_senior");
    assert_eq!(senior.count, 3);
    assert!((senior.area_m2 - 16.5).abs() < 1e-9);
    let day = row(&r, "day_room");
    assert_eq!(day.count, 3);
    assert!((day.area_m2 - 18.0).abs() < 1e-9);
    assert_eq!(day.basis, "estimate");
    let officer = row(&r, "cabin_officer");
    assert_eq!(officer.count, 1);
    assert!((officer.area_m2 - 8.5).abs() < 1e-9);
    assert_eq!(row(&r, "toilet").count, 2);
    assert_eq!(row(&r, "washbasin").count, 8);
    assert!((row(&r, "washbasin").area_m2 - 4.0).abs() < 1e-9);
    assert!((row(&r, "corridor").area_m2 - 7.05).abs() < 1e-9);
    assert!(r.rows.iter().all(|x| x.id != "hospital"));
    assert!((r.total_m2 - 106.27).abs() < 1e-9);
}

#[test]
fn an_officer_cabin_follows_a3_1_9_k_by_gross_tonnage() {
    // Four officers: three have day rooms, the fourth takes A3.1.9(k): 7.5, 8.5, 10 m2.
    for (gt, want) in [(2_999, 7.5), (3_000, 8.5), (10_000, 10.0)] {
        let r = size_rooms(&sea(4, 4, gt, 4));
        let c = row(&r, "cabin_officer");
        assert_eq!(c.count, 1);
        assert!((c.area_each_m2 - want).abs() < 1e-12, "{gt}");
        assert_eq!(c.governing, "mlc");
    }
}

#[test]
fn day_rooms_are_for_at_most_three_officers() {
    // A3.1.9(m): the master, the chief engineer and the chief navigating officer.
    assert_eq!(row(&size_rooms(&sea(5, 1, 5_000, 5)), "day_room").count, 1);
    assert_eq!(row(&size_rooms(&sea(5, 3, 5_000, 5)), "day_room").count, 3);
    let r = size_rooms(&sea(7, 6, 5_000, 7));
    assert_eq!(row(&r, "day_room").count, 3);
    assert_eq!(row(&r, "cabin_officer").count, 3);
    // No officer, no day room.
    assert!(
        size_rooms(&sea(5, 0, 5_000, 5))
            .rows
            .iter()
            .all(|x| x.id != "day_room")
    );
}

#[test]
fn a_washbasin_in_every_sleeping_room_and_the_sanitary_counts() {
    // A3.1.11(d): one washbasin in each sleeping room: 13 rooms = 13 basins (ES-TRIN 15.03(1)(b):
    // ceil(13/4) = 4). A3.1.11(c): one toilet and one shower per six persons: ceil(13/6) = 3 each.
    let r = size_rooms(&sea(13, 3, 5_000, 11));
    let basin = row(&r, "washbasin");
    assert_eq!(basin.count, 13);
    assert_eq!(basin.governing, "mlc");
    assert!(basin.clause.contains("11(d)"));
    assert_eq!(row(&r, "toilet").count, 3);
    assert_eq!(row(&r, "toilet").governing, "both");
    assert_eq!(row(&r, "shower").count, 3);
    assert_eq!(row(&r, "shower").governing, "both");
    // One berth: one basin by either rule.
    assert_eq!(
        row(&size_rooms(&sea(1, 0, 5_000, 1)), "washbasin").governing,
        "both"
    );
}

#[test]
fn a_hospital_from_fifteen_seafarers() {
    // A3.1.12: 15 or more seafarers (voyage over three days; the relaxation for coastal trade and
    // the voyage length are not applied). 15 seafarers: hospital 7.0 and its sanitary 1.0 + 0.5 +
    // 0.81 = 2.31 (Guideline B3.1.8(4)). 14: none.
    let with = size_rooms(&sea(17, 4, 5_000, 15));
    let h = row(&with, "hospital");
    assert_eq!(h.count, 1);
    assert!((h.area_m2 - 7.0).abs() < 1e-9);
    assert_eq!(h.basis, "estimate");
    assert_eq!(h.governing, "mlc");
    assert!((row(&with, "hospital_sanitary").area_m2 - 2.31).abs() < 1e-9);
    let without = size_rooms(&sea(16, 4, 5_000, 14));
    assert!(without.rows.iter().all(|x| !x.id.starts_with("hospital")));
}

#[test]
fn the_sea_rooms_name_their_governing_source_and_say_no_exemption_is_applied() {
    let r = size_rooms(&sea(8, 4, 5_000, 6));
    for x in &r.rows {
        assert!(
            ["es-trin", "mlc", "both"].contains(&x.governing),
            "{}",
            x.id
        );
        assert!((x.area_m2 - x.count as f64 * x.area_each_m2).abs() < 1e-9);
    }
    // The galley, the stairway and the communal room are ES-TRIN's.
    assert_eq!(row(&r, "galley").governing, "es-trin");
    assert_eq!(row(&r, "communal").governing, "es-trin");
    assert!(r.rule_note.contains("No exemption is applied"));
    assert!(r.rule_note.contains("A3.1.20"));
}

#[test]
fn a_sea_hull_is_never_smaller_than_the_river_rooms_for_the_same_berths() {
    for n in [1, 4, 8, 13] {
        let river = size_rooms(&basis(n)).total_m2;
        for gt in [2_999, 5_000, 12_000] {
            assert!(
                size_rooms(&sea(n, 2.min(n), gt, n)).total_m2 > river,
                "{n} {gt}"
            );
        }
    }
}

#[test]
fn no_berths_no_rooms() {
    let r = size_rooms(&basis(0));
    assert_eq!(r.total_m2, 0.0);
    assert!(r.rows.is_empty());
}
