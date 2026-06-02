//! One-shot, source-bound consent tokens for the local Evidence provider (#58).
//!
//! The lawyer's confirmation in the UI causes the backend to ISSUE a short-lived
//! token bound to `(matterId, sourceId, sha256)`. Sending the document to the
//! local model requires CONSUMING that token, which checks the binding and TTL
//! and removes it (one-shot). Nothing is persisted.
//!
//! Threat model (honest): the renderer is OUR bundled code. This token does NOT
//! protect against a fully compromised renderer (which could itself call the
//! issue command then the propose command). It DOES protect against accidental
//! invocations, replay of a stale consent, a consent reused on the WRONG Fonte,
//! and context mismatch (matter/source/version) — turning consent into a
//! backend-verified, single-use, source-bound step rather than a bare boolean.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// How long an issued consent token stays valid.
pub const CONSENT_TTL: Duration = Duration::from_secs(120);

/// Why consuming a consent token failed (generic, no client content).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentError {
    /// Token unknown, already used, or never issued.
    MissingOrUsed,
    /// Token found but past its TTL.
    Expired,
    /// Token does not match the (matter, source, sha256) it is being used for.
    ContextMismatch,
}

impl std::fmt::Display for ConsentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Generic, user-facing, never carries client content.
        match self {
            ConsentError::MissingOrUsed => write!(f, "consenso assente o già utilizzato"),
            ConsentError::Expired => write!(f, "consenso scaduto, riprova"),
            ConsentError::ContextMismatch => write!(f, "consenso non valido per questa Fonte"),
        }
    }
}

#[derive(Clone)]
struct Entry {
    matter_id: String,
    source_id: String,
    sha256: String,
    expires_at: SystemTime,
}

/// In-memory store of outstanding consent tokens. Not persisted; cleared on exit.
#[derive(Default)]
pub struct ConsentStore {
    tokens: HashMap<String, Entry>,
}

impl ConsentStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a freshly-issued `token` bound to `(matter, source, sha256)`,
    /// valid until `now + CONSENT_TTL`.
    pub fn issue(
        &mut self,
        token: &str,
        matter_id: &str,
        source_id: &str,
        sha256: &str,
        now: SystemTime,
    ) {
        self.tokens.insert(
            token.to_string(),
            Entry {
                matter_id: matter_id.to_string(),
                source_id: source_id.to_string(),
                sha256: sha256.to_string(),
                expires_at: now + CONSENT_TTL,
            },
        );
    }

    /// Consume `token` for `(matter, source, sha256)`. One-shot: the token is
    /// removed whatever the outcome (so a wrong/expired token can't be retried).
    /// Returns the matching error otherwise.
    pub fn consume(
        &mut self,
        token: &str,
        matter_id: &str,
        source_id: &str,
        sha256: &str,
        now: SystemTime,
    ) -> Result<(), ConsentError> {
        let entry = self
            .tokens
            .remove(token)
            .ok_or(ConsentError::MissingOrUsed)?;
        if now > entry.expires_at {
            return Err(ConsentError::Expired);
        }
        if entry.matter_id != matter_id || entry.source_id != source_id || entry.sha256 != sha256 {
            return Err(ConsentError::ContextMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t0() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000)
    }

    #[test]
    fn issued_token_is_consumed_once() {
        let mut s = ConsentStore::new();
        s.issue("tok", "m", "src", "sha", t0());
        assert_eq!(s.consume("tok", "m", "src", "sha", t0()), Ok(()));
        // One-shot: a second use fails (replay protection).
        assert_eq!(
            s.consume("tok", "m", "src", "sha", t0()),
            Err(ConsentError::MissingOrUsed)
        );
    }

    #[test]
    fn missing_token_is_rejected() {
        let mut s = ConsentStore::new();
        assert_eq!(
            s.consume("nope", "m", "src", "sha", t0()),
            Err(ConsentError::MissingOrUsed)
        );
    }

    #[test]
    fn expired_token_is_rejected_and_burned() {
        let mut s = ConsentStore::new();
        s.issue("tok", "m", "src", "sha", t0());
        let later = t0() + CONSENT_TTL + Duration::from_secs(1);
        assert_eq!(
            s.consume("tok", "m", "src", "sha", later),
            Err(ConsentError::Expired)
        );
        // Burned even though it failed (no retry after the error).
        assert_eq!(
            s.consume("tok", "m", "src", "sha", t0()),
            Err(ConsentError::MissingOrUsed)
        );
    }

    #[test]
    fn token_is_bound_to_matter_source_and_sha() {
        let mut s = ConsentStore::new();
        s.issue("tok", "m", "src", "sha", t0());
        // Wrong source → mismatch (cross-source reuse blocked).
        assert_eq!(
            s.consume("tok", "m", "other", "sha", t0()),
            Err(ConsentError::ContextMismatch)
        );

        s.issue("tok2", "m", "src", "sha", t0());
        // Wrong sha (document changed/re-imported) → mismatch.
        assert_eq!(
            s.consume("tok2", "m", "src", "different-sha", t0()),
            Err(ConsentError::ContextMismatch)
        );
    }
}
