//! What the shell holds besides the Design session: the file path, the unsaved flag, the active
//! Option (for the window title), and the hash warnings and the notes of a changed old file from the
//! last open.
//!
//! This module has no Tauri types, so it is tested like any other Rust. It holds no calculation,
//! validation or default logic: every answer comes from [`DesignSession`].

use crate::damage::{self, End, Job, Kept, Running};
use crate::{vessel, views};
use ovd_core::text::{Arg, Msg};
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, check, msg_json};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

pub struct AppState {
    session: DesignSession,
    path: Option<PathBuf>,
    dirty: bool,
    active_option: String,
    /// Names of the Options whose stored results hash differs from this engine's, at open.
    hash_changed: Vec<String>,
    /// What changed when an old file was read as the current inputs, at open: one note per change,
    /// kept with the Option it is about, so a rename or a delete of that Option follows (issue #436).
    migration_notes: Vec<Note>,
    /// The run of the Option the 3D view last asked about (issue #457), until that Option changes.
    vessel: Option<vessel::Kept>,
    /// The damage run each Option last had (issue #924), by name. Shown while its fingerprint still
    /// matches the Option's inputs and the Defaults Profile; an edit leaves it unused.
    damage: HashMap<String, Kept>,
    /// The damage run under way, if any.
    running: Option<Running>,
}

impl Drop for AppState {
    /// A state that goes (a new or opened Design replaces it) cancels the run for its Option.
    fn drop(&mut self) {
        self.cancel_damage();
    }
}

/// One migration note: the Option it is about (none for a note about the file), the message without
/// its "Option '<name>': " lead-in, and that message's Message Id.
#[derive(Debug, Clone)]
struct Note {
    option: Option<String>,
    body: String,
    msg: Option<Msg>,
}

impl Note {
    /// A note as the report words it. A note about an Option has the Message Id `msg-option`, with
    /// the Option's name and the message itself as its arguments.
    fn from_report(text: &str, msg: Option<&Msg>) -> Note {
        if let Some(m) = msg.filter(|m| m.id == "msg-option") {
            let arg = |name: &str| m.args.iter().find(|(n, _)| *n == name).map(|(_, a)| a);
            if let (Some(Arg::Text(option)), Some(message)) = (arg("option"), arg("message")) {
                let (body, msg) = match message {
                    Arg::Msg(inner) => (inner.english.clone(), Some(inner.clone())),
                    other => (other.english(), None),
                };
                return Note {
                    option: Some(option.clone()),
                    body,
                    msg,
                };
            }
        }
        Note {
            option: None,
            body: text.to_string(),
            msg: msg.cloned(),
        }
    }

    /// The note's text, with the Option's name as it is now.
    fn text(&self) -> String {
        match &self.option {
            Some(name) => format!("Option '{name}': {}", self.body),
            None => self.body.clone(),
        }
    }

    /// The note's Message Id and arguments, with the Option's name as it is now.
    fn msg(&self) -> Value {
        let Some(body) = &self.msg else {
            return Value::Null;
        };
        match &self.option {
            Some(name) => json!({
                "id": "msg-option",
                "args": { "option": name, "message": msg_json(body, true) },
            }),
            None => msg_json(body, false),
        }
    }
}

fn first_option(session: &DesignSession) -> String {
    session
        .design()
        .options
        .first()
        .map(|o| o.name.clone())
        .unwrap_or_default()
}

impl AppState {
    /// A new Design: the starter Design `ovd new` writes.
    pub fn new_design() -> Result<AppState, String> {
        let session = DesignSession::new_design(Profile::built_in())?;
        let active_option = first_option(&session);
        Ok(AppState {
            session,
            path: None,
            dirty: false,
            active_option,
            hash_changed: Vec::new(),
            migration_notes: Vec::new(),
            vessel: None,
            damage: HashMap::new(),
            running: None,
        })
    }

    /// Open a `.vdesign` text that was read from `path`.
    pub fn open(path: &Path, text: &str) -> Result<AppState, String> {
        let (session, report) = DesignSession::open(text, Profile::built_in())?;
        let active_option = first_option(&session);
        let hash_changed = report
            .hash_status
            .into_iter()
            .filter(|(_, status)| matches!(status, HashStatus::Differs { .. }))
            .map(|(name, _)| name)
            .collect();
        Ok(AppState {
            session,
            path: Some(path.to_path_buf()),
            dirty: false,
            active_option,
            hash_changed,
            migration_notes: report
                .report
                .migration_notes
                .iter()
                .enumerate()
                .map(|(i, text)| {
                    let msg = report
                        .report
                        .migration_note_msgs
                        .get(i)
                        .and_then(Option::as_ref);
                    Note::from_report(text, msg)
                })
                .collect(),
            vessel: None,
            damage: HashMap::new(),
            running: None,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn session(&self) -> &DesignSession {
        &self.session
    }

    /// The kept run of the Option named `option` for the 3D view, run again when the Option changed.
    fn vessel(&mut self, option: &str) -> Result<&vessel::Kept, String> {
        vessel::kept(&mut self.vessel, &self.session.design().options, option)
    }

    /// The mesh vertex data of an Option (issue #457): positions (f32, x y z per vertex) and
    /// indices (u32, three per triangle).
    pub fn mesh_buffers(&mut self, option: &str) -> Result<(Vec<f32>, Vec<u32>), String> {
        self.vessel(option).map(vessel::buffers)
    }

    /// The mesh group table and hash of an Option.
    pub fn mesh_groups(&mut self, option: &str) -> Result<Value, String> {
        self.vessel(option).map(vessel::groups)
    }

    /// The hull's section outline of an Option at `x`.
    pub fn section_at(&mut self, option: &str, x: f64) -> Result<Value, String> {
        self.vessel(option).map(|k| vessel::section(k, x))
    }

    /// A Space's name, kind, volume and centroid.
    pub fn space_info(&mut self, option: &str, name: &str) -> Result<Value, String> {
        self.vessel(option).and_then(|k| vessel::space(k, name))
    }

    /// The Deck Items of an Option and its wheelhouse's lift (spec 0009).
    pub fn deck_items(&mut self, option: &str) -> Result<Value, String> {
        self.vessel(option).map(vessel::deck_items)
    }

    /// The propulsor parts of an Option, for the pick panel (spec 0009).
    pub fn propulsors(&mut self, option: &str) -> Result<Value, String> {
        self.vessel(option).map(vessel::propulsors)
    }

    /// The waterline of each floating position of an Option.
    pub fn waterlines(&mut self, option: &str) -> Result<Value, String> {
        self.vessel(option).map(vessel::waterlines)
    }

    /// The evaluation as JSON for the front end: the session's own, with the hull views (SVG) added
    /// to the outcome of each Option that was computed, and the damage case drawings (SVG) of each
    /// Option whose results hold damage cases.
    pub fn evaluate_json(&self) -> Value {
        let mut evaluation = self.session.evaluate_with_damage(&damage::off());
        // An Option with a damage run for its current inputs shows that run.
        for option in &mut evaluation.options {
            if let (Some(kept), Ok(now)) = (
                self.damage.get(&option.name),
                self.session.option_fingerprint(&option.name),
            ) && kept.fingerprint == now
                && option.outcome.is_some()
            {
                option.outcome = Some(kept.outcome.clone());
            }
        }
        let mut json = evaluation.to_json();
        for (i, option) in evaluation.options.iter().enumerate() {
            if let Some(outcome @ RunOutcome::Results { .. }) = &option.outcome {
                json["options"][i]["outcome"]["views"] = views::hull_views(outcome, &option.name);
                if let Some(damage) = views::damage_views(outcome, &option.name) {
                    json["options"][i]["outcome"]["damage_views"] = damage;
                }
            }
        }
        json
    }

    /// The Design changed: it is unsaved, and a damage run whose Option changed (or went) stops.
    fn edited(&mut self) {
        self.dirty = true;
        let stale = self.running.as_ref().is_some_and(|r| {
            self.session.option_fingerprint(&r.option).ok().as_deref() != Some(&r.fingerprint)
        });
        if stale {
            self.cancel_damage();
        }
        // A kept run of an Option that went is dropped; the others stay until their fingerprint differs.
        let names: Vec<String> = self
            .session
            .design()
            .options
            .iter()
            .map(|o| o.name.clone())
            .collect();
        self.damage.retain(|name, _| names.contains(name));
    }

    pub fn set_input(&mut self, option: &str, name: &str, value: Value) -> Result<(), String> {
        self.session.set_input(option, name, value)?;
        self.edited();
        Ok(())
    }

    /// Issue #451: import an offsets CSV into an Option (see `DesignSession::import_offsets`). A
    /// resampled table's warning shows with the Design's messages.
    pub fn import_offsets(&mut self, option: &str, csv: &str) -> Result<(), String> {
        self.session.import_offsets(option, csv)?;
        self.edited();
        Ok(())
    }

    pub fn reset_input(&mut self, option: &str, name: &str) -> Result<(), String> {
        self.session.reset_input(option, name)?;
        self.edited();
        Ok(())
    }

    pub fn set_formation(
        &mut self,
        option: &str,
        kind: &str,
        abreast: Option<u64>,
        in_line: Option<u64>,
    ) -> Result<(), String> {
        self.session.set_formation(option, kind, abreast, in_line)?;
        self.edited();
        Ok(())
    }

    /// Add an Option and make it the active one.
    pub fn add_option(&mut self) -> Result<(), String> {
        self.active_option = self.session.add_option(None)?;
        self.edited();
        Ok(())
    }

    /// Copy an Option and make the copy the active one.
    pub fn duplicate_option(&mut self, option: &str) -> Result<(), String> {
        self.active_option = self.session.duplicate_option(option, None)?;
        self.edited();
        Ok(())
    }

    pub fn rename_option(&mut self, option: &str, new_name: &str) -> Result<(), String> {
        self.session.rename_option(option, new_name)?;
        let new_name = new_name.trim();
        // The kept damage run, and the run under way, follow the new name.
        if new_name != option {
            if let Some(kept) = self.damage.remove(option) {
                self.damage.insert(new_name.to_string(), kept);
            }
            if let Some(r) = self.running.as_mut().filter(|r| r.option == option) {
                r.option = new_name.to_string();
            }
        }
        if self.active_option == option {
            self.active_option = new_name.to_string();
        }
        // The notes and hash warnings of the Option follow its new name.
        for note in &mut self.migration_notes {
            if note.option.as_deref() == Some(option) {
                note.option = Some(new_name.to_string());
            }
        }
        for name in &mut self.hash_changed {
            if name == option {
                *name = new_name.to_string();
            }
        }
        self.edited();
        Ok(())
    }

    /// Delete an Option. The session refuses to delete the last one.
    pub fn delete_option(&mut self, option: &str) -> Result<(), String> {
        self.session.delete_option(option)?;
        if self.active_option == option {
            self.active_option = first_option(&self.session);
        }
        // The notes and hash warnings of a deleted Option go with it.
        self.migration_notes
            .retain(|n| n.option.as_deref() != Some(option));
        self.hash_changed.retain(|n| n != option);
        self.edited();
        Ok(())
    }

    /// Swap in a Defaults Profile from its text. The session refuses one that loosens a hard range.
    pub fn load_profile(&mut self, text: &str) -> Result<(), String> {
        self.session.load_profile(text)?;
        self.edited();
        Ok(())
    }

    pub fn use_built_in_profile(&mut self) -> Result<(), String> {
        self.load_profile(Profile::built_in_text())
    }

    pub fn set_active_option(&mut self, option: &str) -> Result<(), String> {
        if self
            .session
            .design()
            .options
            .iter()
            .any(|o| o.name == option)
        {
            self.active_option = option.to_string();
            Ok(())
        } else {
            Err(format!("no Option named '{option}'"))
        }
    }

    /// Start a damage run of `option` (issue #924): a copy of the Option for the engine, so the lock
    /// is free while the cases run. One run at a time; the caller runs the [`Job`] off this lock, then
    /// gives its end to [`AppState::finish_damage`]. Refused while the Option has a hard error.
    pub fn damage_job(&mut self, option: &str) -> Result<Job, String> {
        if self.running.is_some() {
            return Err("damage stability is already running".into());
        }
        let fingerprint = self.session.option_fingerprint(option)?;
        let checked = check(&self.session.design().to_text(), self.session.profile());
        if !checked.report.is_ok() {
            return Err(format!(
                "damage stability cannot run while the Design has an error: {}",
                checked.report.errors.join("; ")
            ));
        }
        let run_option = checked
            .design
            .and_then(|d| d.options.into_iter().find(|o| o.name == option))
            .ok_or_else(|| format!("no Option named '{option}'"))?;
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.running = Some(Running {
            option: option.to_string(),
            fingerprint: fingerprint.clone(),
            cancel: cancel.clone(),
        });
        Ok(Job::new(
            option.to_string(),
            fingerprint,
            run_option,
            cancel,
        ))
    }

    /// A damage run ended. Its results are kept only when it finished and the Option's inputs and the
    /// profile are still what it ran for. Returns whether they were kept.
    pub fn finish_damage(&mut self, job: &Job, end: End) -> bool {
        if self
            .running
            .as_ref()
            .is_some_and(|r| std::sync::Arc::ptr_eq(&r.cancel, &job.cancel))
        {
            self.running = None;
        }
        self.keep_damage(&job.option, &job.fingerprint, end)
    }

    /// Keep the results of a damage run done elsewhere (the web build runs it in a worker): `fingerprint`
    /// is the run's, and must be the Option's now.
    pub fn keep_damage(&mut self, option: &str, fingerprint: &str, end: End) -> bool {
        let End::Done(outcome) = end else {
            return false;
        };
        if self.session.option_fingerprint(option).ok().as_deref() != Some(fingerprint) {
            return false;
        }
        self.damage.insert(
            option.to_string(),
            Kept {
                fingerprint: fingerprint.to_string(),
                outcome,
            },
        );
        true
    }

    /// The kept damage run of an Option as JSON, `{ fingerprint, results }`, for a worker that ran it
    /// to hand back (the web build). `None` when the Option has no kept run.
    pub fn damage_run_json(&self, option: &str) -> Option<Value> {
        let kept = self.damage.get(option)?;
        match &kept.outcome {
            RunOutcome::Results { results, .. } => {
                Some(json!({ "fingerprint": kept.fingerprint, "results": results }))
            }
            _ => None,
        }
    }

    /// Stop the damage run under way, if any. The Option stays `not run`.
    pub fn cancel_damage(&mut self) {
        if let Some(r) = self.running.take() {
            r.cancel.store(true, Ordering::SeqCst);
        }
    }

    /// Is a damage run under way, and for which Option?
    pub fn damage_running(&self) -> Option<&str> {
        self.running.as_ref().map(|r| r.option.as_str())
    }

    /// The `.vdesign` text to write. Refused while the Design has a hard error.
    pub fn save_text(&self) -> Result<String, String> {
        self.session.save()
    }

    /// Record that the Design was written to `path`.
    pub fn saved(&mut self, path: &Path) {
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        // A saved file carries fresh hashes and the current inputs, so the open-time warnings are over.
        self.hash_changed.clear();
        self.migration_notes.clear();
    }

    pub fn file_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into())
    }

    /// The window title: unsaved marker, file name, active Option.
    pub fn title(&self) -> String {
        let marker = if self.dirty { "\u{25cf} " } else { "" };
        format!(
            "{marker}{} \u{2014} {} \u{2014} OVD",
            self.file_name(),
            self.active_option
        )
    }

    /// What the front end shows. The Design is the session's own `.vdesign` text, parsed.
    pub fn snapshot(&self) -> Value {
        let design: Value =
            serde_json::from_str(&self.session.design().to_text()).unwrap_or(Value::Null);
        json!({
            "engine_version": ovd_format::vdesign::ENGINE_VERSION,
            "profile": self.session.profile().name(),
            "design": design,
            "file_name": self.file_name(),
            "path": self.path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "dirty": self.dirty,
            "title": self.title(),
            "active_option": self.active_option,
            "hash_changed": self.hash_changed,
            "migration_notes": self.migration_notes.iter().map(Note::text).collect::<Vec<_>>(),
            "migration_note_msgs": self.migration_notes.iter().map(Note::msg).collect::<Vec<_>>(),
            "migration_note_options": self.migration_notes.iter().map(|n| n.option.clone()).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ovd_format::{RunOutcome, check, new_design};

    /// Issue #925: the starter has no Service Area, so a test that runs it sets one first.
    fn started() -> AppState {
        let mut state = AppState::new_design().unwrap();
        state
            .set_input("Option 1", "service_area", serde_json::json!("river-only"))
            .unwrap();
        // Issue #1034: a river-only Option needs a number for both river limits before it runs.
        for (name, v) in [("draught_limit_m", 12.0), ("air_draft_limit_m", 40.0)] {
            state
                .set_input("Option 1", name, serde_json::json!(v))
                .unwrap();
        }
        state.dirty = false;
        state
    }

    fn starter_text() -> String {
        new_design(&Profile::built_in()).unwrap().to_text()
    }

    /// The starter with a river-only Service Area (issue #925), and its results hash.
    fn starter_text_with_area() -> String {
        let text = ovd_format::with_service_area(&starter_text(), "river-only").unwrap();
        // Issue #1034: a river-only Option needs a number for both river limits.
        let text = ovd_format::with_river_limits(&text, Some(12.0), Some(40.0)).unwrap();
        ovd_format::fill(&text, &Profile::built_in())
            .design
            .unwrap()
            .to_text()
    }

    #[test]
    fn new_is_the_starter_design_ovd_new_writes() {
        let state = AppState::new_design().unwrap();
        let expected: Value = serde_json::from_str(&starter_text()).unwrap();
        assert_eq!(state.snapshot()["design"], expected);
        assert_eq!(state.file_name(), "Untitled");
        assert!(!state.is_dirty());
    }

    #[test]
    fn title_shows_file_option_and_the_unsaved_marker() {
        let mut state =
            AppState::open(Path::new("/x/barge.vdesign"), &starter_text_with_area()).unwrap();
        assert_eq!(
            state.title(),
            "barge.vdesign \u{2014} Option 1 \u{2014} OVD"
        );
        state.set_input("Option 1", "beam_m", json!(12.0)).unwrap();
        assert_eq!(
            state.title(),
            "\u{25cf} barge.vdesign \u{2014} Option 1 \u{2014} OVD"
        );
        state.saved(Path::new("/x/other.vdesign"));
        assert_eq!(
            state.title(),
            "other.vdesign \u{2014} Option 1 \u{2014} OVD"
        );
    }

    #[test]
    fn an_edit_marks_the_design_unsaved_and_a_refused_edit_does_not() {
        let mut state = started();
        assert!(
            state
                .set_input("Option 1", "no_such_input", json!(1))
                .is_err()
        );
        assert!(state.set_input("No such", "beam_m", json!(1)).is_err());
        assert!(!state.is_dirty());
        state.reset_input("Option 1", "max_speed_kn").unwrap();
        assert!(state.is_dirty());
    }

    #[test]
    fn active_option_must_exist() {
        let mut state = started();
        assert!(state.set_active_option("Option 9").is_err());
        assert_eq!(state.snapshot()["active_option"], "Option 1");
    }

    #[test]
    fn a_saved_file_passes_validate_and_reopens_unchanged() {
        let mut state = started();
        state.set_input("Option 1", "beam_m", json!(12.5)).unwrap();
        let text = state.save_text().unwrap();
        let checked = check(&text, &Profile::built_in());
        assert!(checked.report.is_ok(), "{:?}", checked.report.errors);
        let reopened = AppState::open(Path::new("a.vdesign"), &text).unwrap();
        assert_eq!(reopened.save_text().unwrap(), text);
        assert!(
            reopened.snapshot()["hash_changed"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_saved_file_carries_each_options_results_hash() {
        let state = started();
        let text = state.save_text().unwrap();
        let v: Value = serde_json::from_str(&text).unwrap();
        let hash = v["options"][0]["results_hash"].as_str().unwrap();
        let eval = state.session().evaluate();
        let Some(RunOutcome::Results { hash: computed, .. }) = &eval.options[0].outcome else {
            panic!("the starter Design should compute");
        };
        assert_eq!(hash, computed);
    }

    #[test]
    fn opening_a_file_whose_results_hash_changed_names_the_options_affected() {
        let mut v: Value = serde_json::from_str(&starter_text_with_area()).unwrap();
        v["options"][0]["results_hash"] = json!("0000000000000000deadbeef");
        let state = AppState::open(Path::new("old.vdesign"), &v.to_string()).unwrap();
        assert_eq!(state.snapshot()["hash_changed"], json!(["Option 1"]));
        // Saving writes fresh hashes, which ends the warning.
        let mut state = state;
        state.saved(Path::new("old.vdesign"));
        assert_eq!(state.snapshot()["hash_changed"], json!([]));
    }

    #[test]
    fn opening_a_version_1_adn_g_file_shows_what_changed_until_it_is_saved() {
        // Issue #175: a version 1 file that names ADN_G opens as IGC_2G, and the window says so.
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/ovd-format/tests/legacy/v1_adn_g_self_propelled.vdesign"
        );
        // Issue #925: the legacy file has no Service Area; these tests are about its migration notes.
        let text =
            ovd_format::with_service_area(&std::fs::read_to_string(path).unwrap(), "river-only")
                .unwrap();
        let mut state = AppState::open(Path::new("old.vdesign"), &text).unwrap();
        let notes = state.snapshot()["migration_notes"].clone();
        let notes = notes.as_array().unwrap();
        // Three for the version 1 changes, one for the dome diameter (issue #237: the dome now
        // counts in Vc, so results may change), and one for the hull end input a version 1 file
        // lacks (issue #4): it gets the fixed value the engine used. One more for the LPG carriage
        // condition the file lacks (issue #292): it is read as fully pressurised. One more for the
        // block coefficient the file stores (issue #448): it is no longer an input. One more for the
        // structure block of the Hull the file lacks (issue #739). One more for the cargo vent mast
        // inputs the file lacks (issue #822). One more for the ADN stack a river-only gas file is read with
        // (issue #850).
        assert_eq!(notes.len(), 10, "{notes:?}");
        assert!(
            notes[0]
                .as_str()
                .unwrap()
                .contains("ADN_G is no longer offered")
        );
        assert!(
            notes[1]
                .as_str()
                .unwrap()
                .contains("`adn_tank_cap_m3` is no longer an input")
        );
        assert!(
            notes[2]
                .as_str()
                .unwrap()
                .contains("has no `max_tank_od_m`; it now uses the default 16 m")
        );
        assert!(
            notes[3].as_str().unwrap().contains(
                "`block_coefficient`: the block coefficient now comes from the hull shape"
            ),
            "{notes:?}"
        );
        assert!(
            notes[4]
                .as_str()
                .unwrap()
                .contains("has no `tank_dome_diameter_m`; it now holds 2.0 m"),
            "{notes:?}"
        );
        assert!(
            notes[5]
                .as_str()
                .unwrap()
                .contains("has no `lpg_carriage_condition`"),
            "{notes:?}"
        );
        // Issue #822: the vent mast inputs have a note of their own, and the results move.
        assert!(
            notes[6]
                .as_str()
                .unwrap()
                .contains("cargo vent mast inputs"),
            "{notes:?}"
        );
        // Issue #850: the ADN stack a river-only gas file is read with has a note of its own.
        assert!(
            notes[7].as_str().unwrap().contains("ADN 2025 9.3.1 stack"),
            "{notes:?}"
        );
        // Issue #739: the structure block's own note comes before the note of the fixed values.
        assert!(
            notes[8]
                .as_str()
                .unwrap()
                .contains("before the structure block of the Hull"),
            "{notes:?}"
        );
        let last = notes[9].as_str().unwrap();
        assert!(
            last.contains("has no `")
                && last.contains("`bow_rake_length_m`")
                && last.contains("`bow_rake_length_to_depth_ratio`"),
            "{last}"
        );
        assert!(
            last.contains("fixed value the engine used before it was an input"),
            "{last}"
        );
        // Issue #274: the ruleset is no longer an input; the name is read and dropped.
        let ruleset = &state.snapshot()["design"]["options"][0]["inputs"]["ruleset"];
        assert!(ruleset.is_null());
        // The saved file is the current format and needs no notes.
        state.saved(Path::new("old.vdesign"));
        assert_eq!(state.snapshot()["migration_notes"], json!([]));
        let saved: Value = serde_json::from_str(&state.save_text().unwrap()).unwrap();
        assert_eq!(saved["format_version"], 4);
        // A file that needs no change has none.
        let fresh = AppState::open(Path::new("a.vdesign"), &starter_text_with_area()).unwrap();
        assert_eq!(fresh.snapshot()["migration_notes"], json!([]));
    }

    #[test]
    fn an_options_migration_notes_follow_a_rename_and_go_with_a_delete() {
        // Issue #436: a note stayed on screen after its Option was deleted.
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/ovd-format/tests/legacy/v1_adn_g_self_propelled.vdesign"
        );
        // Issue #925: the legacy file has no Service Area; these tests are about its migration notes.
        let text =
            ovd_format::with_service_area(&std::fs::read_to_string(path).unwrap(), "river-only")
                .unwrap();
        let mut state = AppState::open(Path::new("old.vdesign"), &text).unwrap();
        let notes = |state: &AppState| {
            state.snapshot()["migration_notes"]
                .as_array()
                .unwrap()
                .clone()
        };
        let before = notes(&state);
        // Six as above, one for the stored block coefficient (issue #448), one for the structure
        // block (issue #739), one for the vent mast inputs (issue #822) and one for the ADN stack (issue #850).
        assert_eq!(before.len(), 10);
        state.duplicate_option("Option 1").unwrap();
        assert_eq!(notes(&state), before, "a copy brings no notes of its own");

        state.rename_option("Option 1", "Laden case").unwrap();
        let after = notes(&state);
        assert_eq!(after.len(), 10);
        for (old, new) in before.iter().zip(&after) {
            let old = old.as_str().unwrap();
            let new = new.as_str().unwrap();
            assert_eq!(
                new,
                old.replacen("Option 'Option 1': ", "Option 'Laden case': ", 1)
            );
        }
        let snap = state.snapshot();
        for (i, msg) in snap["migration_note_msgs"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            assert_eq!(msg["id"], "msg-option");
            assert_eq!(msg["args"]["option"], "Laden case", "note {i}");
        }
        assert!(
            snap["migration_note_options"]
                .as_array()
                .unwrap()
                .iter()
                .all(|o| o == "Laden case")
        );

        state.delete_option("Laden case").unwrap();
        assert_eq!(
            notes(&state),
            Vec::<Value>::new(),
            "the notes went with their Option"
        );
        assert_eq!(state.snapshot()["migration_note_msgs"], json!([]));
    }

    #[test]
    fn a_hash_warning_follows_a_rename_and_goes_with_a_delete() {
        let mut state = started();
        state.hash_changed = vec!["Option 1".into()];
        state.duplicate_option("Option 1").unwrap();
        state.rename_option("Option 1", "B").unwrap();
        assert_eq!(state.snapshot()["hash_changed"], json!(["B"]));
        state.delete_option("B").unwrap();
        assert_eq!(state.snapshot()["hash_changed"], json!([]));
    }

    #[test]
    fn a_class_pick_reaches_the_front_end_with_its_reference_deadweight() {
        // Issue #94: the class the vessel sits in and its reference deadweight are in the
        // evaluation, beside the outcome and not in the results.
        let mut state = started();
        assert!(state.evaluate_json()["options"][0]["vessel_class"].is_null());
        state
            .set_input("Option 1", "vessel_class", json!("CEMT-Va"))
            .unwrap();
        let eval = state.evaluate_json();
        let class = &eval["options"][0]["vessel_class"];
        assert_eq!(class["id"], "CEMT-Va");
        assert_eq!(class["reference_dwt_t"], json!([1500.0, 3000.0]));
        assert_eq!(
            state.snapshot()["design"]["options"][0]["inputs"]["length_overall_m"]["source"],
            "class"
        );
        assert!(state.is_dirty());
    }

    #[test]
    fn option_commands_keep_the_active_option_in_step() {
        let mut state = started();
        state.add_option().unwrap();
        assert_eq!(state.snapshot()["active_option"], "Option 2");
        state.rename_option("Option 2", "  Wide ").unwrap();
        assert_eq!(state.snapshot()["active_option"], "Wide");
        state.duplicate_option("Wide").unwrap();
        assert_eq!(state.snapshot()["active_option"], "Wide copy");
        state.delete_option("Wide copy").unwrap();
        assert_eq!(state.snapshot()["active_option"], "Option 1");
        state.delete_option("Wide").unwrap();
        assert!(state.delete_option("Option 1").is_err());
        assert!(state.rename_option("Option 1", "").is_err());
    }

    #[test]
    fn a_profile_that_loosens_a_hard_range_is_refused_and_the_old_one_stays() {
        let mut v: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
        v["name"] = json!("loose");
        let text = serde_json::to_string(&v).unwrap();
        let mut state = started();
        state.load_profile(&text).unwrap();
        assert_eq!(state.snapshot()["profile"], "loose");
        state.use_built_in_profile().unwrap();
        assert_eq!(state.snapshot()["profile"], "built-in");
        v["inputs"]["beam_m"]["hard"]["le"] = json!(5000);
        let err = state.load_profile(&v.to_string()).unwrap_err();
        assert!(err.contains("loosens the hard range of `beam_m`"), "{err}");
        assert!(state.load_profile("not json").is_err());
        assert_eq!(state.snapshot()["profile"], "built-in");
    }

    #[test]
    fn the_evaluation_carries_the_three_hull_views_of_a_computed_option() {
        let state = started();
        let eval = state.evaluate_json();
        let outcome = &eval["options"][0]["outcome"];
        assert_eq!(outcome["status"], "results");
        for key in ["side", "plan", "midship"] {
            assert!(
                outcome["views"][key].as_str().unwrap().contains("<svg"),
                "{key}"
            );
        }
        // the rest of the evaluation is the session's own
        assert_eq!(eval["options"][0]["name"], "Option 1");
        assert!(outcome["results"]["dims"].is_object());
    }

    #[test]
    fn the_evaluation_carries_the_damage_svg_for_damage_results_and_none_otherwise() {
        let state = started();
        let eval = state.evaluate_json();
        assert!(eval["options"][0]["outcome"].get("damage_views").is_none());

        // Issue #924: an evaluation runs no damage, so the drawings come after a damage run.
        let mut state = gas_state();
        assert!(
            state.evaluate_json()["options"][0]["outcome"]
                .get("damage_views")
                .is_none()
        );
        run_damage(&mut state);
        let eval = state.evaluate_json();
        let outcome = &eval["options"][0]["outcome"];
        assert_eq!(outcome["status"], "results");
        let sheets = outcome["damage_views"]["sheets"].as_array().unwrap();
        assert_eq!(
            sheets.len(),
            outcome["results"]["damage"]["governing"]
                .as_array()
                .unwrap()
                .len()
        );
        assert!(sheets[0]["svg"].as_str().unwrap().contains("<svg"));
    }

    #[test]
    fn an_option_renamed_to_a_wide_name_keeps_its_three_views() {
        // Issue #660: the Drawings tab of "140 x 29 x 9m LNG" showed a clause 14.6 message.
        let mut state = started();
        state
            .rename_option("Option 1", "140 x 29 x 9m LNG")
            .unwrap();
        let eval = state.evaluate_json();
        let outcome = &eval["options"][0]["outcome"];
        assert!(
            outcome["views"].get("message").is_none(),
            "{}",
            outcome["views"]
        );
        for key in ["side", "plan", "midship"] {
            assert!(
                outcome["views"][key].as_str().unwrap().contains("<svg"),
                "{key}"
            );
        }
    }

    #[test]
    fn a_name_too_wide_for_the_drawings_is_refused_when_it_is_given() {
        let mut state = started();
        let name = "A very long descriptive Option name that no title block can hold";
        let err = state.rename_option("Option 1", name).unwrap_err();
        assert!(
            err.contains("too wide for the drawing title block"),
            "{err}"
        );
        assert_eq!(state.session().design().options[0].name, "Option 1");
    }

    #[test]
    fn an_option_with_a_hard_error_gets_no_views() {
        let mut state = started();
        state.set_input("Option 1", "beam_m", json!(-1.0)).unwrap();
        let eval = state.evaluate_json();
        assert_eq!(eval["options"][0]["outcome"]["status"], "not_run");
        assert!(eval["options"][0]["outcome"].get("views").is_none());
    }

    #[test]
    fn a_file_that_is_not_a_design_is_refused() {
        assert!(AppState::open(Path::new("x.vdesign"), "not json").is_err());
    }

    // ---- Issue #924: damage on demand ----

    /// A coastal gas hull with damage cases (the IGC cases are in the coastal-and-river gas stack),
    /// opened as the app opens a file.
    fn gas_state() -> AppState {
        let text = views::tests::gas_damage_design_text();
        AppState::open(Path::new("/x/gas.vdesign"), &text).unwrap()
    }

    fn results_of(state: &AppState) -> Value {
        state.evaluate_json()["options"][0]["outcome"]["results"].clone()
    }

    /// Run damage on the first Option and keep it, as the desktop command does.
    fn run_damage(state: &mut AppState) -> (Vec<(usize, usize)>, bool) {
        let name = state.session().design().options[0].name.clone();
        let job = state.damage_job(&name).unwrap();
        let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let log = calls.clone();
        let end = job.run(move |done, total| log.borrow_mut().push((done, total)));
        let kept = state.finish_damage(&job, end);
        let calls = calls.borrow().clone();
        (calls, kept)
    }

    /// Issue #924: only a run the user asks for works damage out. Every path of the app that reaches the
    /// engine runs without it: open, evaluate, save and the drawing and 3D calls.
    #[test]
    fn open_evaluate_and_save_run_no_damage() {
        let text = views::tests::gas_damage_design_text();
        let before = ovd_format::damage::damage_runs_started();
        let state = AppState::open(Path::new("/x/gas.vdesign"), &text).unwrap();
        assert_eq!(
            ovd_format::damage::damage_runs_started(),
            before,
            "open ran damage"
        );
        let runs = || ovd_format::damage::damage_runs_started();
        state.evaluate_json();
        assert_eq!(runs(), before, "evaluate ran damage");
        state.save_text().unwrap();
        assert_eq!(runs(), before, "save ran damage");
        let mut state = state;
        let name = state.session().design().options[0].name.clone();
        state.mesh_buffers(&name).unwrap();
        assert_eq!(runs(), before, "the 3D mesh ran damage");
        state.waterlines(&name).unwrap();
        state.deck_items(&name).unwrap();
        assert_eq!(
            ovd_format::damage::damage_runs_started(),
            before,
            "evaluate or save ran damage"
        );
        // The counter does move when damage runs, so the check above can fail.
        run_damage(&mut state);
        assert!(ovd_format::damage::damage_runs_started() > before);
    }

    #[test]
    fn an_evaluation_runs_no_damage_until_it_is_asked_for() {
        let mut state = gas_state();
        let before = results_of(&state);
        assert_eq!(before["damage_status"], "not run");
        let total = before["damage_cases"].as_u64().unwrap();
        assert!(total > 4, "{total} damage cases");
        assert!(before.get("damage").is_none_or(|d| d.is_null()));

        let (calls, kept) = run_damage(&mut state);
        assert!(kept);
        // Issue #935: the search runs damage on each candidate that passes stability, then the answer's run
        // follows, so the count is cumulative: it starts at 0, never falls, and ends with every case done. The
        // total the no-damage run gave counts the unscreened arrangement's cases; the last total holds the search's
        // runs as well as the answer's.
        assert_eq!(calls.first().map(|c| c.0), Some(0));
        assert!(calls.windows(2).all(|w| w[0].0 <= w[1].0), "done fell");
        // The total is planned and shrinks at the end: the fraction done never falls.
        assert!(
            calls
                .windows(2)
                .all(|w| w[0].0 as f64 / w[0].1 as f64 <= w[1].0 as f64 / w[1].1 as f64 + 1e-12),
            "the fraction fell"
        );
        let last = *calls.last().unwrap();
        assert_eq!(last.0, last.1);
        let after = results_of(&state);
        assert_eq!(after["damage_status"], "run");
        let cases = after["damage_cases"].as_u64().unwrap();
        assert!(cases > 0);
        assert!(last.1 as u64 >= cases, "{last:?} against {cases} cases");
        // The first evaluation did not screen by damage and said so; the run did.
        assert_eq!(before["damage_screening"]["screened"], false);
        assert!(before["damage_screening"]["note"].is_string());
        assert_eq!(after["damage_screening"]["screened"], true);
        let same_pick = before["arrangement"]["tank_od_m"] == after["arrangement"]["tank_od_m"]
            && before["arrangement"]["n_wide"] == after["arrangement"]["n_wide"];
        // The cases are those of the arrangement the run picked: equal to the first count only if the pick stayed.
        if same_pick {
            assert_eq!(cases, total);
            assert!(after["damage_screening"]["changed_note"].is_null());
        } else {
            assert!(after["damage_screening"]["changed_note"].is_string());
            assert_eq!(
                after["damage_screening"]["stage_one"]["tank_od_m"],
                before["arrangement"]["tank_od_m"]
            );
        }
        assert!(
            state.evaluate_json()["options"][0]["outcome"]["damage_views"].is_object(),
            "the damage drawings come from the kept run"
        );
    }

    #[test]
    fn an_edit_or_a_new_profile_makes_the_kept_run_not_run_again() {
        let mut state = gas_state();
        run_damage(&mut state);
        assert_eq!(results_of(&state)["damage_status"], "run");
        // A rename keeps the run.
        let first = state.session().design().options[0].name.clone();
        state.rename_option(&first, "Coastal").unwrap();
        let name = state.session().design().options[0].name.clone();
        assert_eq!(results_of(&state)["damage_status"], "run");
        // An edit does not.
        state.set_input(&name, "beam_m", json!(29.5)).unwrap();
        assert_eq!(results_of(&state)["damage_status"], "not run");

        // A profile of the same name with other content makes a run not run.
        let mut state = gas_state();
        run_damage(&mut state);
        let mut profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
        let (_, entry) = profile["inputs"]
            .as_object_mut()
            .unwrap()
            .iter_mut()
            .next()
            .unwrap();
        entry["basis"] = json!("a changed basis");
        state.load_profile(&profile.to_string()).unwrap();
        assert_eq!(results_of(&state)["damage_status"], "not run");
    }

    #[test]
    fn a_kept_run_follows_a_rename_and_goes_with_a_delete() {
        let mut state = gas_state();
        run_damage(&mut state);
        let first = state.session().design().options[0].name.clone();
        state.rename_option(&first, "Renamed").unwrap();
        assert_eq!(results_of(&state)["damage_status"], "run");
        assert!(state.damage.contains_key("Renamed"));
        state.add_option().unwrap();
        state.delete_option("Renamed").unwrap();
        assert!(state.damage.is_empty());
    }

    #[test]
    fn save_writes_the_hash_of_the_stage_it_shows() {
        let mut state = gas_state();
        let without = state.save_text().unwrap();
        // Saving again changes nothing: the hash is stable for the same stage.
        assert_eq!(state.save_text().unwrap(), without);
        let before = results_of(&state);
        run_damage(&mut state);
        let after = results_of(&state);
        // Issue #935: damage runs the search with the IGC 2.7 screen, so the stage that ran damage may pick
        // another arrangement than the first evaluation. A save works without damage (issue #924), so it writes
        // the hash of the first stage whatever the kept run shows: stable. The hash shown with the kept run is
        // the saved one exactly when the pick did not move.
        let same_pick = before["arrangement"] == after["arrangement"];
        let saved_with: Value = serde_json::from_str(&state.save_text().unwrap()).unwrap();
        let saved_without: Value = serde_json::from_str(&without).unwrap();
        assert_eq!(
            saved_with["options"][0]["results_hash"],
            saved_without["options"][0]["results_hash"]
        );
        // The save says which stage its hash belongs to (issue #935).
        assert_eq!(saved_with["options"][0]["results_stage"], "unscreened");
        let shown = &state.evaluate_json()["options"][0]["outcome"]["results_hash"];
        assert_eq!(
            *shown == saved_with["options"][0]["results_hash"],
            same_pick
        );
        // A second save of the stage that ran damage is stable too.
        assert_eq!(state.save_text().unwrap(), state.save_text().unwrap());
    }

    #[test]
    fn a_stopped_run_keeps_nothing() {
        let mut state = gas_state();
        let name = state.session().design().options[0].name.clone();
        let job = state.damage_job(&name).unwrap();
        // One run at a time.
        assert!(state.damage_job(&name).is_err());
        let stop = job.canceller();
        let end = job.run(move |done, _| {
            if done >= 3 {
                stop.store(true, Ordering::SeqCst);
            }
        });
        assert_eq!(end, End::Cancelled);
        assert!(!state.finish_damage(&job, end));
        assert_eq!(results_of(&state)["damage_status"], "not run");
        assert!(state.damage_running().is_none());
    }

    #[test]
    fn an_edit_cancels_the_run_of_its_option() {
        let mut state = gas_state();
        let name = state.session().design().options[0].name.clone();
        let job = state.damage_job(&name).unwrap();
        state.set_input(&name, "beam_m", json!(29.5)).unwrap();
        assert!(job.canceller().load(Ordering::SeqCst));
        assert_eq!(job.run(|_, _| {}), End::Cancelled);
        // A new Design in its place cancels too.
        let mut state = gas_state();
        let job = state.damage_job(&name).unwrap();
        drop(state);
        assert!(job.canceller().load(Ordering::SeqCst));
    }
}
