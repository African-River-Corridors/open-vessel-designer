//! The crew model of spec 0011 section 4.1 (issue #778): the minimum crew of a self-propelled hull
//! from a manning Rule Set, the owner margin, the spare berths, and the UK cross-check.
//!
//! Concept and preliminary design only: never a Safe Manning Document (ADR 0011). The tables are the
//! data of the manning Rule Sets (`liberia-man-001`, `uk-msn-1868`, `ccnr-rpn-19`); this module holds
//! only the method that reads them. Every step is inference resting on a cited paragraph, and each
//! Rule Set is a draft until a second reader verifies it. A number here is not a rule unless the
//! results say which paragraph it came from.
//!
//! The method, in the order of the spec:
//! 1. the regime from the Service Area picks the Rule Set (`seagoing` Liberia; `inland` CCNR RPN 19);
//! 2. the bridge watches from the daily navigation hours, the deck officers floored by the flag table;
//! 3. the engineers and ratings from the Liberia column picked by GT, power and UMS, a hole in the
//!    grid filled from the nearest column (see [`select_liberia_column`]);
//! 4. the cook (seagoing, minimum of 10 or more);
//! 5. the owner margin, apart from the minimum; the berths = the crew + 2 spare berths (D16).

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// The manning Rule Sets the crew model runs on (spec 0011 D13). Held in step with
/// `ovd_format::vdesign::MANNING_RULE_SETS` by a test.
pub const MANNING_RULE_SETS: [&str; 3] = ["liberia-man-001", "uk-msn-1868", "ccnr-rpn-19"];

/// Spare berths above the crew: a pilot and one supernumerary (spec 0011 D16). An estimate.
pub const SPARE_BERTHS: u32 = 2;

/// More than this many navigation hours in 24 puts three watchkeepers on the bridge (spec 0011 4.1
/// step 3, resting on STCW A-VIII/1 rest; MSN 1868 and MAN-001 give no hours). An estimate.
pub const THREE_WATCH_HOURS: f64 = 12.0;

/// Above this gross tonnage the master keeps no regular watch: Liberia MAN-001, 3.2.1 ("except for
/// ships of 3,000 GT or less").
pub const MASTER_WATCH_FREE_GT: f64 = 3000.0;

/// A seagoing manning of this many persons or more takes a ship's cook: Liberia MAN-001, 12.1.9 and
/// note 5 to the table of 12.2.
pub const COOK_AT_MANNING: u32 = 10;

/// The role ids of a column of the Liberia table (12.2), in the order of the table. The tables of the
/// Rule Set are `liberia-<role id>`.
pub const LIBERIA_ROLES: [&str; 10] = [
    "master",
    "chief-mate",
    "oow",
    "ab-deck",
    "rating-nav",
    "chief-engineer",
    "second-engineer",
    "eoow",
    "ab-engine",
    "rating-engine",
];

/// The role ids of a cell of the RPN tables, in the order of the table rows. The tables of the Rule
/// Set are `ccnr-rpn-19.0x-<name>` with these names.
pub const RPN_ROLES: [(&str, &str); 6] = [
    ("boatmaster", "boatmaster"),
    ("helmsman", "helmsman"),
    ("able boatman", "able-boatman"),
    ("boatman", "boatman"),
    ("apprentice", "apprentice"),
    ("engineer", "engineer"),
];

/// One column of the Liberia MAN-001 table of 12.2, as the Rule Set holds it.
#[derive(Debug, Clone, PartialEq)]
pub struct LiberiaColumn {
    /// The column id of the Rule Set (`c1` to `c13`, in the order of the table).
    pub id: String,
    /// The GT band: 0 under 500, 1 from 500 to 3,000, 2 from 3,000 to 5,000, 3 from 5,000 to 8,000,
    /// 4 over 8,000.
    pub gt_band: u32,
    /// The kW band: 0 from 750 to 3,000, 1 over 3,000.
    pub kw_band: u32,
    /// The column is for a class notation for unattended machinery spaces.
    pub ums: bool,
    /// The column is for the coastwise route only; the others are for every route.
    pub coastwise: bool,
    /// The persons of each role, in the order of [`LIBERIA_ROLES`].
    pub counts: [u32; 10],
}

impl LiberiaColumn {
    /// The persons of the column, the cook not counted (as the table's own total).
    pub fn total(&self) -> u32 {
        self.counts.iter().sum()
    }
}

/// The column the crew model took, and why.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnChoice {
    /// The index in the columns given.
    pub index: usize,
    /// The grid has a column for the GT band, the kW band and the UMS state.
    pub exact: bool,
    /// For a hole: the column that keeps the GT band and takes the next kW band up, if there is one.
    /// The choice does not take it; it is the sensitivity the results show.
    pub same_gt_band: Option<usize>,
}

/// The Liberia column for a GT band, a kW band and a UMS state. The route is not an input of the
/// model, so a coastwise column (the small-ship columns) is never taken: the other routes' columns
/// need at least as many persons, the cautious reading.
///
/// The grid has holes: no column for GT over 8,000 or from 5,000 to 8,000 with 750 to 3,000 kW, none
/// for GT under 500 on every route, none without UMS at 750 to 3,000 kW. A hole takes the
/// neighbour that keeps the **kW band** (the power is what the engine room is manned for) and moves
/// the GT band to the nearest one that has a column (the higher band on a tie); with none that keeps
/// the kW band (a non-UMS ship of 750 to 3,000 kW), the column of the same GT band at the next kW
/// band up, then the nearest of the rest, the higher total on a tie. The same-GT-band column is kept
/// as `same_gt_band` for the results to show.
pub fn select_liberia_column(
    columns: &[LiberiaColumn],
    gt_band: u32,
    kw_band: u32,
    ums: bool,
) -> Option<ColumnChoice> {
    let eligible: Vec<usize> = (0..columns.len())
        .filter(|&i| !columns[i].coastwise && columns[i].ums == ums)
        .collect();
    if let Some(&i) = eligible
        .iter()
        .find(|&&i| columns[i].gt_band == gt_band && columns[i].kw_band == kw_band)
    {
        return Some(ColumnChoice {
            index: i,
            exact: true,
            same_gt_band: None,
        });
    }
    let same_gt_band = eligible
        .iter()
        .copied()
        .filter(|&i| columns[i].gt_band == gt_band)
        .max_by_key(|&i| columns[i].total());
    // The hole keeps the kW band and moves the GT band the least; on a tie the higher GT band.
    let by_kw = eligible
        .iter()
        .copied()
        .filter(|&i| columns[i].kw_band == kw_band)
        .min_by_key(|&i| {
            let d = columns[i].gt_band.abs_diff(gt_band);
            (d, u32::MAX - columns[i].gt_band)
        });
    let index = by_kw.or(same_gt_band).or_else(|| {
        eligible.iter().copied().min_by_key(|&i| {
            let c = &columns[i];
            (
                c.gt_band.abs_diff(gt_band) + c.kw_band.abs_diff(kw_band),
                u32::MAX - c.total(),
            )
        })
    })?;
    Some(ColumnChoice {
        index,
        exact: false,
        same_gt_band: same_gt_band.filter(|&i| i != index),
    })
}

/// The persons of one role of a crew.
#[derive(Debug, Clone, PartialEq)]
pub struct Role {
    /// The role id (`master`, `chief-mate`, `oow`, ... or an RPN function).
    pub role: String,
    pub count: u32,
}

fn role(role: &str, count: u32) -> Role {
    Role {
        role: role.to_string(),
        count,
    }
}

/// The bridge watch arrangement from the daily navigation hours and the gross tonnage (spec 0011 4.1
/// step 3).
#[derive(Debug, Clone, PartialEq)]
pub struct Watches {
    /// Persons who stand bridge watches: three above [`THREE_WATCH_HOURS`] hours, else two.
    pub watchkeepers: u32,
    /// The master keeps no regular watch (above [`MASTER_WATCH_FREE_GT`]).
    pub master_watch_free: bool,
    /// The deck officers the watches need: the master and the watchkeepers when the master is watch
    /// free, else the watchkeepers (the master stands one).
    pub deck_officers: u32,
}

/// The watches of a hull (spec 0011 4.1 steps 3 and 4).
pub fn watches(gt: f64, navigation_hours: f64) -> Watches {
    let watchkeepers = if navigation_hours > THREE_WATCH_HOURS {
        3
    } else {
        2
    };
    let master_watch_free = gt > MASTER_WATCH_FREE_GT;
    Watches {
        watchkeepers,
        master_watch_free,
        deck_officers: if master_watch_free {
            watchkeepers + 1
        } else {
            watchkeepers
        },
    }
}

/// A seagoing minimum crew from a Liberia column and the watches.
#[derive(Debug, Clone, PartialEq)]
pub struct SeagoingCrew {
    pub roles: Vec<Role>,
    pub watches: Watches,
    /// Master, chief mate and officers of the watch (after the floor of the watches).
    pub deck_officers: u32,
    /// Chief engineer, second engineer and engineering watch officers.
    pub engineers: u32,
    /// The able seafarers and the ratings of both watches.
    pub ratings: u32,
    /// The ship's cook: 1 when the persons above reach [`COOK_AT_MANNING`], else 0.
    pub cook: u32,
    /// The minimum: deck officers, engineers, ratings and the cook.
    pub minimum: u32,
    /// Deck officers the watches need beyond the flag table's (added to the officers of the watch).
    pub added_by_watches: u32,
}

/// The seagoing minimum crew on a Liberia column (spec 0011 4.1 steps 3 to 8).
///
/// `cook_applies`: the hull reaches the sea, so MLC 2006 and its cook rule apply; a river-only hull on
/// a seagoing scale takes no cook.
pub fn seagoing_crew(
    column: &LiberiaColumn,
    gt: f64,
    navigation_hours: f64,
    cook_applies: bool,
) -> SeagoingCrew {
    let w = watches(gt, navigation_hours);
    let c = &column.counts;
    let table_deck = c[0] + c[1] + c[2];
    let added = w.deck_officers.saturating_sub(table_deck);
    let mut roles: Vec<Role> = LIBERIA_ROLES
        .iter()
        .zip(c.iter())
        .map(|(r, n)| role(r, *n))
        .collect();
    // The watches ask for more than the table: the extra persons are officers of the watch.
    roles[2].count += added;
    let deck_officers = table_deck + added;
    let engineers = c[5] + c[6] + c[7];
    let ratings = c[3] + c[4] + c[8] + c[9];
    let before_cook = deck_officers + engineers + ratings;
    let cook = u32::from(cook_applies && before_cook >= COOK_AT_MANNING);
    SeagoingCrew {
        roles,
        watches: w,
        deck_officers,
        engineers,
        ratings,
        cook,
        minimum: before_cook + cook,
        added_by_watches: added,
    }
}

/// One cell (or one alternative of a cell) of the RPN 19.02 or 19.03 table, as the Rule Set holds it.
#[derive(Debug, Clone, PartialEq)]
pub struct RpnCell {
    /// `g<group>-<mode>-<standard>`, with `-alt<n>` where the table offers alternatives.
    pub key: String,
    /// The persons of each function, in the order of [`RPN_ROLES`].
    pub counts: [u32; 6],
}

impl RpnCell {
    pub fn total(&self) -> u32 {
        self.counts.iter().sum()
    }

    /// The `g<group>-<mode>-<standard>` part of the key, without the alternative.
    fn base(&self) -> &str {
        match self.key.find("-alt") {
            Some(i) => &self.key[..i],
            None => &self.key,
        }
    }
}

/// The RPN operating mode for the daily navigation hours (RPN 18.01(1)): A1 up to 14 hours, A2 up to
/// 18, B up to 24. More than 24 hours is no mode: B, flagged by the caller.
pub fn rpn_mode(mode_index: u32) -> &'static str {
    match mode_index {
        1 => "A1",
        2 => "A2",
        _ => "B",
    }
}

/// An inland minimum crew from an RPN table.
#[derive(Debug, Clone, PartialEq)]
pub struct InlandCrew {
    /// The cell taken: the greatest crew of every cell of the group and mode (both equipment
    /// standards, every alternative), the cautious reading.
    pub key: String,
    pub roles: Vec<Role>,
    pub minimum: u32,
    /// The smallest crew of the group and mode, for the results to show beside it.
    pub least: u32,
}

/// The minimum crew of a group and mode: the cell with the most persons of the group and mode, over
/// both equipment standards and every alternative of the table (RPN 19.02 and 19.03). `None` when the
/// table has no such group and mode.
pub fn inland_crew(cells: &[RpnCell], group: u32, mode: &str) -> Option<InlandCrew> {
    let prefix = alloc::format!("g{group}-{mode}-");
    let of_group: Vec<&RpnCell> = cells
        .iter()
        .filter(|c| c.key.starts_with(&prefix))
        .collect();
    let best = of_group.iter().copied().max_by_key(|c| c.total())?;
    let least = of_group.iter().map(|c| c.total()).min()?;
    let roles = RPN_ROLES
        .iter()
        .zip(best.counts.iter())
        .map(|((name, _), n)| role(name, *n))
        .collect();
    Some(InlandCrew {
        key: best.base().to_string(),
        roles,
        minimum: best.total(),
        least,
    })
}

/// The owner margin of a hull (spec 0011 4.1 step 9 and 7): persons the owner may add over the
/// minimum, reported apart from it. Estimates; none is a rule.
///
/// Seagoing: a cargo engineer on a gas carrier (step 7), a steward and an electro-technical officer.
/// Inland: an engineer on a gas hull, where the RPN prescribes none.
pub fn owner_margin(seagoing: bool, gas_cargo: bool) -> Vec<Role> {
    let mut v = Vec::new();
    if seagoing {
        if gas_cargo {
            v.push(role("cargo-engineer", 1));
        }
        v.push(role("steward", 1));
        v.push(role("electro-technical-officer", 1));
    } else if gas_cargo {
        v.push(role("engineer", 1));
    }
    v
}

/// The persons aboard and the berths (spec 0011 4.1 step 11 and D16).
#[derive(Debug, Clone, PartialEq)]
pub struct Berths {
    /// The persons used: the typed crew, else the minimum plus the whole owner margin.
    pub crew: u32,
    /// The crew plus [`SPARE_BERTHS`].
    pub berths: u32,
    /// The planning range without a typed crew: the minimum plus the first margin item, and the
    /// minimum plus all of the margin.
    pub planning: (u32, u32),
}

/// The crew used and the berths from the minimum, the owner margin and a typed crew.
pub fn berths(minimum: u32, margin: &[Role], typed: Option<u32>) -> Berths {
    let all: u32 = margin.iter().map(|r| r.count).sum();
    let first = margin.first().map_or(0, |r| r.count);
    let crew = typed.unwrap_or(minimum + all);
    Berths {
        crew,
        berths: crew + SPARE_BERTHS,
        planning: (minimum + first, minimum + all),
    }
}

/// The UK MSN 1868 cross-check (spec 0011 4.1 step 10): officers only, as the UK guidance gives no
/// ratings.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossCheck {
    /// Deck officers and engineers of the UK guidance.
    pub uk_officers: u32,
    /// Deck officers and engineers of the Liberia run.
    pub liberia_officers: u32,
    /// UK less Liberia.
    pub difference: i64,
    /// The two differ by more than one person.
    pub flagged: bool,
}

/// Compare the officers of the UK guidance with the Liberia run: a difference over one person is
/// flagged.
pub fn cross_check(uk_officers: u32, liberia_officers: u32) -> CrossCheck {
    let difference = i64::from(uk_officers) - i64::from(liberia_officers);
    CrossCheck {
        uk_officers,
        liberia_officers,
        difference,
        flagged: difference.abs() > 1,
    }
}
