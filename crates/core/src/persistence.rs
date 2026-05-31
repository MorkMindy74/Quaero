//! Canonical JSON (de)serialization boundary for local persistence (#5B).
//!
//! Pure and Tauri-free (ADR-0011): no filesystem, no paths, no I/O here — only
//! the canonical wire <-> type mapping. Filesystem I/O lives in the desktop
//! crate. The ONLY persistable shape is the canonical [`Workspace`]:
//!
//! - [`to_json`] takes `&Workspace` by type, so a derived `WorkspaceView` can
//!   never be persisted through this boundary (it is not the accepted type).
//! - [`from_json`] is a thin wrapper over `serde_json::from_str::<Workspace>`,
//!   so loading always flows through `RawWorkspace` + `TryFrom`: referential
//!   integrity, the reserved `dyn-` rejection, and `deny_unknown_fields` (which
//!   rejects a shadow top-level `dossiers`) all apply to persisted state.

use crate::domain::Workspace;

/// Serialize a canonical workspace to pretty, human-diffable JSON.
pub fn to_json(workspace: &Workspace) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(workspace)
}

/// Parse a canonical workspace from JSON, enforcing the full #5A contract via
/// the `RawWorkspace` + `TryFrom` deserialization path.
pub fn from_json(json: &str) -> Result<Workspace, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::sample_workspace;

    #[test]
    fn round_trip_preserves_canonical_workspace() {
        let ws = sample_workspace();
        let json = to_json(&ws).unwrap();
        let back = from_json(&json).unwrap();
        assert_eq!(ws, back);
    }

    #[test]
    fn serialized_json_is_canonical_no_view_no_dynamic() {
        let json = to_json(&sample_workspace()).unwrap();
        // canonical key present…
        assert!(json.contains("manualDossiers"));
        // …and no derived/view state leaks into persisted JSON.
        assert!(!json.contains("\"dossiers\""));
        assert!(!json.contains("dyn-"));
        assert!(!json.contains("\"Dynamic\""));
    }

    #[test]
    fn from_json_rejects_shadow_top_level_dossiers() {
        // A saved WorkspaceView (carrying derived `dossiers`) must not load as
        // a canonical Workspace: deny_unknown_fields rejects it.
        let json = r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"a","title":"t","subject":"s"},"sources":[],"manualDossiers":[],"dossiers":[]}"#;
        assert!(from_json(json).is_err());
    }

    #[test]
    fn from_json_rejects_incoherent_graph() {
        // client/matter mismatch
        let mismatch = r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"b","title":"t","subject":"s"},"sources":[],"manualDossiers":[]}"#;
        assert!(from_json(mismatch).is_err());
        // dangling manual source reference
        let dangling = r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"a","title":"t","subject":"s"},"sources":[],"manualDossiers":[{"id":"man-x","name":"X","sources":["ghost"]}]}"#;
        assert!(from_json(dangling).is_err());
        // reserved dyn- manual id
        let dyn_id = r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"a","title":"t","subject":"s"},"sources":[],"manualDossiers":[{"id":"dyn-x","name":"X","sources":[]}]}"#;
        assert!(from_json(dyn_id).is_err());
    }

    #[test]
    fn to_json_round_trips_through_view_free_canonical_only() {
        // Saving then loading yields an equal canonical Workspace whose derived
        // view still computes dynamic dossiers (proving we persisted canonical
        // state, not the view).
        let ws = sample_workspace();
        let reloaded = from_json(&to_json(&ws).unwrap()).unwrap();
        assert_eq!(ws, reloaded);
        assert!(reloaded
            .view()
            .dossiers
            .iter()
            .any(|d| d.name == "Documenti"));
    }
}
