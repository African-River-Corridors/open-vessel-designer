//! The accommodation rooms of issue #780 (spec 0011 D5, section 2), sized from ES-TRIN 2025/1
//! Chapter 15 (the text read in the rule sources: `es-trin/ES_TRIN_2025_signed_en.txt`, pages 103-107).
//!
//! Label: golden values, each worked by hand from the paragraph quoted beside it, not read back from
//! the code.

use ovd_core::rooms::*;

fn basis(berths: u32) -> RoomsBasis {
    RoomsBasis {
        berths,
        reaches_sea: false,
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
    // total 21.00 + 12.00 + 1.00 + 3.15 = 37.15
    let r = size_rooms(&basis(6));
    assert!((row(&r, "cabin").area_m2 - 21.0).abs() < 1e-9);
    assert!((row(&r, "communal").area_m2 - 12.0).abs() < 1e-9);
    assert_eq!(row(&r, "toilet").count, 1);
    assert!((row(&r, "toilet").area_m2 - 1.0).abs() < 1e-9);
    assert!((row(&r, "corridor").area_m2 - 3.15).abs() < 1e-9);
    assert!((r.total_m2 - 37.15).abs() < 1e-9);
}

#[test]
fn a_small_crew_takes_the_8_m2_communal_floor() {
    // 1 berth: 2 m2 is under the 8 m2 total of 15.02(8): 8.00. Cabin 3.50, toilet 1.00,
    // corridor 0.60 x (1 bay x 1.75) = 1.05. Total 13.55.
    let r = size_rooms(&basis(1));
    assert!((row(&r, "communal").area_m2 - 8.0).abs() < 1e-9);
    assert!((row(&r, "corridor").area_m2 - 1.05).abs() < 1e-9);
    assert!((r.total_m2 - 13.55).abs() < 1e-9);
}

#[test]
fn thirteen_berths_and_the_fixture_counts() {
    // cabins 13 x 3.5 = 45.5; communal 26.0; toilets ceil(13/6) = 3 -> 3.00;
    // corridor 0.6 x (7 bays x 1.75) = 7.35; total 81.85.
    // Basins ceil(13/4) = 4 and showers ceil(13/6) = 3 (15.03(1)(b), (c)); the text gives no floor
    // area for them, so they are counted and their area is a named gap (0).
    let r = size_rooms(&basis(13));
    assert!((r.total_m2 - 81.85).abs() < 1e-9);
    assert_eq!(row(&r, "washbasin").count, 4);
    assert_eq!(row(&r, "shower").count, 3);
    assert_eq!(row(&r, "washbasin").basis, "gap");
    assert_eq!(row(&r, "shower").area_m2, 0.0);
}

#[test]
fn every_row_has_a_clause_and_a_basis() {
    for n in [1, 4, 13] {
        for row in &size_rooms(&basis(n)).rows {
            assert!(row.clause.starts_with("ES-TRIN 2025/1 "), "{}", row.id);
            assert!(
                ["sourced", "derived", "gap"].contains(&row.basis),
                "{}",
                row.id
            );
            assert!((row.area_m2 - row.count as f64 * row.area_each_m2).abs() < 1e-9);
        }
    }
}

#[test]
fn the_gaps_are_named_rows_with_no_area() {
    // The galley (15.04(2)), the work-clothes room (15.07(2)) and the stairs (15.02(12)) have
    // contents or widths in the text but no floor area.
    let r = size_rooms(&basis(6));
    for id in ["galley", "work_clothes", "stairs"] {
        let g = row(&r, id);
        assert_eq!((g.basis, g.area_m2), ("gap", 0.0), "{id}");
    }
}

#[test]
fn mode_b_rest_underway_cites_the_one_cabin_each_rule() {
    let b = RoomsBasis {
        berths: 4,
        reaches_sea: false,
        mode_b: true,
    };
    let r = size_rooms(&b);
    assert_eq!(row(&r, "cabin").count, 4);
    assert!(row(&r, "cabin").clause.contains("RPN 18.02(4)"));
    assert!(!row(&size_rooms(&basis(4)), "cabin").clause.contains("RPN"));
}

#[test]
fn a_sea_going_hull_is_sized_to_es_trin_and_says_mlc_was_not_applied() {
    let b = RoomsBasis {
        berths: 6,
        reaches_sea: true,
        mode_b: false,
    };
    let r = size_rooms(&b);
    assert!((r.total_m2 - 37.15).abs() < 1e-9);
    assert!(!r.mlc_applied);
    assert!(r.rule_note.contains("MLC"));
}

#[test]
fn no_berths_no_rooms() {
    let r = size_rooms(&basis(0));
    assert_eq!(r.total_m2, 0.0);
    assert!(r.rows.is_empty());
}
