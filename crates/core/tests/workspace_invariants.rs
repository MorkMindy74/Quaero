//! Integration tests for the canonical `Workspace` referential invariants,
//! exercised through the public crate API only (an external consumer).

use quaero_core::domain::{Client, ClientId, Matter, MatterId, Workspace, WorkspaceError};

fn client(id: &str) -> Client {
    Client {
        id: ClientId::new(id),
        name: id.to_uppercase(),
    }
}

fn matter(id: &str, client: &str) -> Matter {
    Matter {
        id: MatterId::new(id),
        client: ClientId::new(client),
        title: "t".to_string(),
        subject: "s".to_string(),
    }
}

#[test]
fn workspace_new_rejects_client_matter_mismatch() {
    assert_eq!(
        Workspace::new(client("alfa"), matter("m", "beta"), vec![], vec![]),
        Err(WorkspaceError::ClientMismatch)
    );
}

#[test]
fn workspace_new_accepts_a_coherent_graph() {
    assert!(Workspace::new(client("alfa"), matter("m", "alfa"), vec![], vec![]).is_ok());
}

#[test]
fn deserializing_incoherent_workspace_is_rejected() {
    // matter.client (b) does not match client.id (a).
    let json = r#"{"client":{"id":"a","name":"A"},"matter":{"id":"m","client":"b","title":"t","subject":"s"},"sources":[],"manualDossiers":[]}"#;
    assert!(serde_json::from_str::<Workspace>(json).is_err());
}
