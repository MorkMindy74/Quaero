//! Integration tests: exercise the domain ONLY through the public crate API,
//! so they catch invariant bypasses that in-module unit tests cannot (the test
//! module is a child of `domain` and can touch private fields; this crate is an
//! external consumer and cannot).
//!
//! Note: a literal `ManualDossier { id: "dyn-x".into(), .. }` here would not even
//! compile, because the fields are private. The only public way in is `new()`
//! (checked) or serde (validating `TryFrom`). These tests pin that contract.

use quaero_core::domain::{sample_workspace, ManualDossier, ManualDossierError, Workspace};

#[test]
fn manual_dossier_new_rejects_reserved_dyn_prefix() {
    assert_eq!(
        ManualDossier::new("dyn-documento", "X", vec![]),
        Err(ManualDossierError::ReservedDynamicPrefix)
    );
}

#[test]
fn manual_dossier_new_accepts_a_normal_id() {
    let dossier = ManualDossier::new("man-x", "X", vec![]).expect("normal id is valid");
    assert_eq!(dossier.id(), "man-x");
    assert_eq!(dossier.name(), "X");
    assert!(dossier.sources().is_empty());
}

#[test]
fn loaded_manual_dossier_with_dyn_prefix_is_rejected() {
    let json = r#"{
        "client": {"id":"alfa","name":"Alfa"},
        "matter": {"id":"m","client":"alfa","title":"t","subject":"s"},
        "sources": [],
        "manualDossiers": [{"id":"dyn-documento","name":"X","sources":[]}]
    }"#;
    assert!(
        serde_json::from_str::<Workspace>(json).is_err(),
        "a loaded manual dossier must not use the reserved dyn- prefix"
    );
}

#[test]
fn sample_workspace_view_has_unique_dossier_ids() {
    let view = sample_workspace().view();
    let total = view.dossiers.len();
    let mut ids: Vec<&str> = view.dossiers.iter().map(|d| d.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "dossier view ids must be unique");
}

#[test]
fn canonical_workspace_built_via_public_api_never_serializes_a_dynamic_dossier() {
    // The only public path produces valid manual dossiers; the canonical wire
    // shape therefore carries no DossierKind and no dyn- ids.
    let json = serde_json::to_string(&sample_workspace()).unwrap();
    assert!(json.contains("manualDossiers"));
    assert!(!json.contains("\"Dynamic\""));
    assert!(!json.contains("dyn-"));
}
