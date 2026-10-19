//! M0-T65 — the calibration file, its physics hash and the refusal (contract §2.11, §3.1, §3.3, §5.4's G-CAL).
//! No test here touches a GPU: the module lives in the one test binary (R11) so the scope runs through `tests/gpu.sh`.

use serde_json::{json, Value};
use sr_engine::headless::{calibration_gate, run_captured};
use sr_physics::calibration::{
    canonical_json, physics_hash, physics_hash_of, resolve, resolve_against, sha256_hex, Calibration, CalibrationError,
    EMBEDDED, EXIT_CALIBRATION,
};

const ELEMENTS: &str = include_str!("../../../../assets/elements.json");
const REACTIONS: &str = include_str!("../../../../assets/reactions.json");
const PHYSICS: &str = include_str!("../../../../assets/physics.json");

/// The embedded assets' physics hash, recorded here while there is no `assets/calibration.json` to hold it (M0-V9 writes
/// it). `embedded_calibration_matches_the_assets` reads that file once it exists, and this constant then goes unused.
/// A physics, elements or reactions change before then is G-CAL working: re-pin this to the hash the failure prints.
const PINNED_HASH: &str = "556e3959cee177c48914a95ac94d66ea21849dd7866b50bf2691b3137da1f306";

const C_SB: f64 = 200.0;

fn hash() -> String {
    physics_hash().expect("the shipped assets are JSON")
}

#[test]
fn sha256_fips_180_4_short_vectors() {
    // FIPS 180-4 / NIST CAVP: the empty message, "abc" and the one-block 448-bit message.
    assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn sha256_fips_180_4_long_vectors_and_the_padding_edges() {
    // The 896-bit message (two blocks) and one million 'a'.
    assert_eq!(
        sha256_hex(b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"),
        "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"
    );
    assert_eq!(sha256_hex(&vec![b'a'; 1_000_000]), "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
    // 55, 56 and 64 bytes straddle the length field's room in a block: the digests differ and each is 64 hex digits.
    let digests: Vec<String> = [55usize, 56, 57, 63, 64, 65].iter().map(|&n| sha256_hex(&vec![b'x'; n])).collect();
    for (i, d) in digests.iter().enumerate() {
        assert_eq!(d.len(), 64);
        assert!(!digests[..i].contains(d), "a padding edge repeated a digest");
    }
}

#[test]
fn the_hash_is_over_the_canonical_bytes() {
    // The canonical text of a small value, written out by hand: keys sorted, no whitespace, numbers as f64.
    let v: Value = serde_json::from_str(r#" { "b" : [ 1 , 2.50 , "x" ] , "a" : { "z" : true , "y" : null , "w" : 1e3 } } "#).unwrap();
    assert_eq!(canonical_json(&v), r#"{"a":{"w":1000.0,"y":null,"z":true},"b":[1.0,2.5,"x"]}"#);

    // The physics hash is the SHA-256 of the object {"elements","physics","reactions"}, each canonical.
    let tiny = (r#"{"e":1}"#, r#"{"r":[2]}"#, r#"{"p":0.5}"#);
    let bytes = r#"{"elements":{"e":1.0},"physics":{"p":0.5},"reactions":{"r":[2.0]}}"#;
    assert_eq!(physics_hash_of(tiny.0, tiny.1, tiny.2).unwrap(), sha256_hex(bytes.as_bytes()));

    // And over the real files: the three texts as shipped give the hash the crate embeds.
    assert_eq!(physics_hash_of(ELEMENTS, REACTIONS, PHYSICS).unwrap(), hash());
}

#[test]
fn a_reformatted_physics_json_keeps_its_hash_and_a_changed_constant_does_not() {
    let keep = hash();
    let value: Value = serde_json::from_str(PHYSICS).unwrap();
    let object = value.as_object().unwrap();
    // Reformatted: compact, then pretty with the keys reversed, then each number respelled (2 → 2.0 → 2e0).
    let compact = serde_json::to_string(&value).unwrap();
    let mut reversed = String::from("{\n");
    for (i, (k, v)) in object.iter().rev().enumerate() {
        reversed.push_str(&format!("{}      {k:?}   :   {v}", if i > 0 { ",\n" } else { "" }));
    }
    reversed.push_str("\n\n}\n");
    assert_ne!(compact, PHYSICS, "the reformatting must change the bytes");
    assert_ne!(reversed, PHYSICS);
    for text in [compact.as_str(), reversed.as_str()] {
        assert_eq!(physics_hash_of(ELEMENTS, REACTIONS, text).unwrap(), keep, "same physics, same hash");
    }
    assert_eq!(
        physics_hash_of(r#"{"e":1}"#, "[]", r#"{"k":2}"#).unwrap(),
        physics_hash_of(r#"{ "e" : 1.0 }"#, "[ ]", r#"{"k":2e0}"#).unwrap(),
        "2, 2.0 and 2e0 are one number"
    );

    // A changed constant, a new key, a removed key, a changed element and a changed reaction each change it.
    let mut other = value.clone();
    other["paint_t"] = json!(value["paint_t"].as_f64().unwrap() + 1.0);
    assert_ne!(physics_hash_of(ELEMENTS, REACTIONS, &other.to_string()).unwrap(), keep);
    other = value.clone();
    other["extra"] = json!(1);
    assert_ne!(physics_hash_of(ELEMENTS, REACTIONS, &other.to_string()).unwrap(), keep);
    other = value.clone();
    other.as_object_mut().unwrap().remove("c_sb");
    assert_ne!(physics_hash_of(ELEMENTS, REACTIONS, &other.to_string()).unwrap(), keep);
    assert_ne!(physics_hash_of(&ELEMENTS.replacen("Hydrogen", "Hydrogene", 1), REACTIONS, PHYSICS).unwrap(), keep);
    assert_ne!(physics_hash_of(ELEMENTS, &REACTIONS.replacen("H_burn", "H_bUrn", 1), PHYSICS).unwrap(), keep);
    // Not JSON: refused, naming the file.
    match physics_hash_of(ELEMENTS, REACTIONS, "{") {
        Err(CalibrationError::Invalid { key, .. }) => assert_eq!(key, "physics.json"),
        other => panic!("expected a refusal naming physics.json, got {other:?}"),
    }
}

/// The test calibration, as a file's text, with the keys the case changes.
fn file_with(edit: impl FnOnce(&mut Value)) -> String {
    let mut v: Value = serde_json::from_str(&Calibration::test_calibration(&hash()).to_json()).unwrap();
    edit(&mut v);
    v.to_string()
}

fn invalid_key(text: &str) -> String {
    match Calibration::parse(text) {
        Err(CalibrationError::Invalid { key, .. }) => key,
        other => panic!("expected an invalid calibration, got {other:?}"),
    }
}

#[test]
fn the_schema_is_parsed_and_every_refusal_names_its_key() {
    let good = Calibration::test_calibration(&hash());
    let text = good.to_json();
    assert_eq!(Calibration::parse(&text).expect("the test calibration is valid"), good, "a round trip");
    // The file carries exactly §2.11's keys.
    let v: Value = serde_json::from_str(&text).unwrap();
    let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    let mut want = vec![
        "version", "physics_hash", "measured", "m_ch_sb", "m_tov_sb", "ns_sigma_max_sb", "m_ign_sb", "m_up_sb", "m_bh_sb",
        "preset_mass_sb", "t_eff_sun_sb", "life_sun_sb", "top_rung", "stage_durations_sb", "render_reserve_ms", "max_gas_speed",
    ];
    want.sort_unstable();
    assert_eq!(keys, want);

    assert_eq!(invalid_key(&file_with(|v| v["surprise"] = json!(1))), "surprise", "an unknown key");
    assert_eq!(invalid_key(&file_with(|v| v["measured"]["colour"] = json!("x"))), "measured.colour");
    assert_eq!(invalid_key(&file_with(|v| { v.as_object_mut().unwrap().remove("m_ch_sb"); })), "m_ch_sb", "a missing key");
    assert_eq!(invalid_key(&file_with(|v| { v["measured"].as_object_mut().unwrap().remove("box"); })), "measured.box");
    assert_eq!(invalid_key(&file_with(|v| v["version"] = json!(2))), "version");
    assert_eq!(invalid_key(&file_with(|v| v["physics_hash"] = json!("abc"))), "physics_hash");
    assert_eq!(invalid_key(&file_with(|v| v["physics_hash"] = json!("A".repeat(64)))), "physics_hash", "lowercase only");
    assert_eq!(invalid_key(&file_with(|v| v["life_sun_sb"] = json!(0))), "life_sun_sb");
    assert_eq!(invalid_key(&file_with(|v| v["top_rung"] = json!("fast"))), "top_rung");
    assert_eq!(invalid_key(&file_with(|v| v["render_reserve_ms"] = json!(-0.1))), "render_reserve_ms");
    assert_eq!(invalid_key(&file_with(|v| v["preset_mass_sb"]["giant"] = json!(-1))), "preset_mass_sb.giant");
    assert_eq!(invalid_key(&file_with(|v| v["measured"]["git"] = json!(""))), "measured.git");
    assert_eq!(invalid_key(&file_with(|v| v["stage_durations_sb"]["main_sequence"] = json!([]))), "stage_durations_sb.main_sequence");
    assert_eq!(invalid_key(&file_with(|v| v["stage_durations_sb"]["main_sequence"] = json!([[1.0]]))), "stage_durations_sb.main_sequence");
    assert_eq!(invalid_key("not json"), "<json>");
    // The reserve may be 0; nothing else may.
    Calibration::parse(&file_with(|v| v["render_reserve_ms"] = json!(0))).expect("a zero reserve is valid");
}

#[test]
fn the_order_of_the_ceilings_is_k9() {
    // m_ign < m_ch < m_up < m_bh, m_ch < m_tov, m_tov / m_ch in [1.4, 2.0], the presets increasing.
    assert_eq!(invalid_key(&file_with(|v| v["m_ign_sb"] = json!(4000))), "m_ign_sb");
    assert_eq!(invalid_key(&file_with(|v| v["m_up_sb"] = json!(3000))), "m_up_sb");
    assert_eq!(invalid_key(&file_with(|v| v["m_bh_sb"] = json!(26000))), "m_bh_sb");
    assert_eq!(invalid_key(&file_with(|v| v["m_tov_sb"] = json!(3500))), "m_tov_sb");
    assert_eq!(invalid_key(&file_with(|v| v["m_tov_sb"] = json!(3970.0 * 1.39))), "m_tov_sb", "the ratio under 1.4");
    assert_eq!(invalid_key(&file_with(|v| v["m_tov_sb"] = json!(3970.0 * 2.01))), "m_tov_sb", "the ratio over 2.0");
    assert_eq!(invalid_key(&file_with(|v| v["preset_mass_sb"]["massive"] = json!(1000))), "preset_mass_sb.massive");
    Calibration::parse(&file_with(|v| v["m_tov_sb"] = json!(3970.0 * 1.4))).expect("the ratio's edge is inside");
    Calibration::parse(&file_with(|v| v["m_tov_sb"] = json!(3970.0 * 2.0))).expect("the ratio's edge is inside");
}

#[test]
fn a_calibration_written_from_the_embedded_assets_is_accepted() {
    let text = Calibration::test_calibration(&hash()).to_json();
    let cal = resolve(Some(&text), true).expect("its hash is the assets'").expect("a run that needs one gets it");
    assert_eq!(cal.physics_hash, hash());
    assert_eq!(cal.max_gas_speed, 20.0);
    // The same through the gate headless runs; a scene with no preset needs none.
    assert_eq!(calibration_gate(Some(&text), true), Ok(()));
    assert_eq!(calibration_gate(Some(&text), false), Ok(()));
}

#[test]
fn a_mismatched_calibration_is_refused_with_exit_6() {
    // One physics.json constant changed after the calibration was written: its hash is no longer the assets'.
    let mut drifted: Value = serde_json::from_str(PHYSICS).unwrap();
    drifted["paint_t"] = json!(drifted["paint_t"].as_f64().unwrap() + 1.0);
    let stale = Calibration::test_calibration(&physics_hash_of(ELEMENTS, REACTIONS, &drifted.to_string()).unwrap()).to_json();

    let err = resolve(Some(&stale), true).expect_err("a stale calibration is refused");
    assert!(matches!(err, CalibrationError::OutOfDate { .. }), "{err:?}");
    assert_eq!(err.exit_code(), 6);
    assert_eq!(EXIT_CALIBRATION, 6);
    let line = err.line();
    assert!(line.starts_with("SR-ERROR calibration out of date"), "{line}");
    assert!(line.contains(&hash()), "the line names the assets' hash: {line}");
    assert_eq!(calibration_gate(Some(&stale), true), Err((6, line)));

    // The same file written on the other side: the assets stand in for a drifted physics.json.
    let fresh = Calibration::test_calibration(&hash()).to_json();
    let drifted_hash = physics_hash_of(ELEMENTS, REACTIONS, &drifted.to_string()).unwrap();
    assert!(matches!(
        resolve_against(Some(&fresh), true, &drifted_hash, C_SB),
        Err(CalibrationError::OutOfDate { .. })
    ));

    // The refusal belongs to runs that read calibration: a scene with no preset runs on a stale file (calibrate must),
    // and one that needs a calibration and has none is refused as missing.
    assert_eq!(calibration_gate(Some(&stale), false), Ok(()));
    assert_eq!(resolve(Some(&stale), false), Ok(None));
    assert_eq!(resolve(None, false), Ok(None));
    let missing = resolve(None, true).expect_err("a preset needs a calibration");
    assert_eq!((missing.exit_code(), &missing), (6, &CalibrationError::Missing));
    assert!(missing.line().starts_with("SR-ERROR calibration missing"));

    // c_sb must exceed twice the fastest gas the calibration saw (§2.11 item 6): equal is refused.
    let fast = Calibration::test_calibration(&hash());
    assert!(fast.check_against(&hash(), 2.0 * fast.max_gas_speed).is_err());
    assert!(fast.check_against(&hash(), 2.0 * fast.max_gas_speed + 0.01).is_ok());
    assert_eq!(invalid_key(&file_with(|v| v["max_gas_speed"] = json!(0))), "max_gas_speed");
    match resolve(Some(&file_with(|v| v["max_gas_speed"] = json!(C_SB))), true) {
        Err(CalibrationError::Invalid { key, .. }) => assert_eq!(key, "max_gas_speed"),
        other => panic!("a gas as fast as c_sb must be refused, got {other:?}"),
    }
}

#[test]
fn the_calibration_the_binary_embeds_matches_the_assets() {
    // G-CAL itself. Once M0-V9 writes assets/calibration.json and embeds it, that file's physics_hash must equal the
    // assets' — a constant changed without calibrating again is exactly this red. Until the file exists, the hash is
    // pinned above (NOT PROVEN against a real calibration: a pin stands in for it).
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/calibration.json");
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            assert!(EMBEDDED.is_some(), "assets/calibration.json exists but the engine does not embed it (calibration.rs)");
            let cal = Calibration::parse(&text).expect("assets/calibration.json is §2.11's");
            assert_eq!(cal.physics_hash, hash(), "calibration.json is out of date: change a constant, calibrate again");
            assert_eq!(EMBEDDED, Some(text.as_str()), "the embedded text is the file");
        }
        Err(_) => {
            println!("NOT PROVEN (no assets/calibration.json yet): the assets' hash against its pin");
            assert!(EMBEDDED.is_none());
            assert_eq!(hash(), PINNED_HASH, "an asset changed with no calibration to hold the hash: re-pin to {}", hash());
        }
    }
}

#[test]
fn headless_runs_without_a_calibration_when_the_scene_names_no_preset() {
    // The step never reads calibration (G-WATCH): a scene of plain gas runs with none (the CPU twin keeps the case off
    // the GPU), and with none embedded yet a preset scene runs too — exit 6 end to end waits for an embedded file
    // (NOT PROVEN until M0-V9 embeds one; the gate's verdicts are proven above).
    assert!(EMBEDDED.is_none() || calibration_gate(EMBEDDED, false).is_ok());
    assert_eq!(calibration_gate(None, false), Ok(()), "no calibration, no preset: runs");
    assert_eq!(calibration_gate(None, true), Ok(()), "nothing embedded yet: nothing to check");
    let scene = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes/fill_world.json");
    let args: Vec<String> =
        ["--scene", scene.to_str().unwrap(), "--world", "64x64", "--steps", "1", "--cpu-reference"].map(String::from).to_vec();
    let (code, lines) = run_captured(&args);
    assert_eq!(code, 0, "{lines:?}");
    assert!(lines.last().is_some_and(|l| l.starts_with("SR-HEADLESS DONE steps=1")), "{lines:?}");
}
