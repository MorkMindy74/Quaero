//! Privacy Guard (#10). Pure and Tauri-free (ADR-0011).
//!
//! A **transversal privacy boundary**: the mandatory chokepoint that any future
//! egress path (a real LLM provider, OSINT connector, export, network log) MUST
//! consult before client-confidential or user content leaves the device.
//!
//! Today nothing egresses (the chat is an offline stub; no network). So this
//! module defines the **contract and the default** — `evaluate` is a real,
//! tested decision function with a **default-deny** stance — it is simply not
//! wired to any egress yet, because no egress exists. When a future slice adds
//! real egress it MUST route through [`PrivacyPolicy::evaluate`] and turn the
//! UI posture line into a derived/conditional one.
//!
//! v1: no persistence, no consent toggle, no PII redaction.

/// Sensitivity class of a piece of data considered for egress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClass {
    /// Client/matter identity, source titles/meta, document content/bytes,
    /// excerpt quotes, manual dossiers — never leaves the device by default.
    ClientConfidential,
    /// Content the user typed (e.g. a chat prompt).
    UserContent,
    /// Non-sensitive app/UI metadata, anonymous counts.
    NonSensitive,
}

/// Where data would be processed/sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    /// A model running locally on the device.
    LocalModel,
    /// The local on-disk store.
    LocalPersistence,
    /// Rendering inside the app.
    InAppDisplay,
    /// A remote/cloud model service.
    RemoteModel,
    /// An external connector (OSINT, web service, …).
    ExternalConnector,
}

impl Destination {
    /// Whether the destination stays on the device.
    pub fn is_local(&self) -> bool {
        match self {
            Destination::LocalModel | Destination::LocalPersistence | Destination::InAppDisplay => {
                true
            }
            Destination::RemoteModel | Destination::ExternalConnector => false,
        }
    }
}

/// A request to move some data of a given class to a destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EgressRequest {
    pub data_class: DataClass,
    pub destination: Destination,
}

/// Why an egress was denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenyReason {
    /// Client-confidential data may not leave the device.
    ConfidentialToExternal,
    /// User content may not leave the device.
    UserContentToExternal,
}

impl std::fmt::Display for DenyReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DenyReason::ConfidentialToExternal => {
                write!(f, "client-confidential data must not leave the device")
            }
            DenyReason::UserContentToExternal => {
                write!(f, "user content must not leave the device")
            }
        }
    }
}

/// The guard's decision for an [`EgressRequest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allowed,
    Denied(DenyReason),
}

/// The privacy stance. In v1 there is a single stance — **local-only** — with no
/// configurable knobs (no consent/toggle that does nothing). The type is the
/// deliberate seam through which a future, explicitly-authorized consent model
/// will flow; until then the stance is fixed and `evaluate` is a pure function
/// of the request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrivacyPolicy;

impl PrivacyPolicy {
    /// Decide whether `request` is permitted. Local destinations are always
    /// allowed; to a non-local destination only `NonSensitive` data may go —
    /// client-confidential and user content are denied (default-deny).
    pub fn evaluate(&self, request: &EgressRequest) -> Decision {
        if request.destination.is_local() {
            return Decision::Allowed;
        }
        match request.data_class {
            DataClass::NonSensitive => Decision::Allowed,
            DataClass::ClientConfidential => Decision::Denied(DenyReason::ConfidentialToExternal),
            DataClass::UserContent => Decision::Denied(DenyReason::UserContentToExternal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL: [Destination; 3] = [
        Destination::LocalModel,
        Destination::LocalPersistence,
        Destination::InAppDisplay,
    ];
    const NON_LOCAL: [Destination; 2] = [Destination::RemoteModel, Destination::ExternalConnector];

    fn decide(data_class: DataClass, destination: Destination) -> Decision {
        PrivacyPolicy.evaluate(&EgressRequest {
            data_class,
            destination,
        })
    }

    #[test]
    fn is_local_classifies_destinations() {
        for d in LOCAL {
            assert!(d.is_local(), "{d:?} should be local");
        }
        for d in NON_LOCAL {
            assert!(!d.is_local(), "{d:?} should be non-local");
        }
    }

    #[test]
    fn local_destinations_always_allowed_for_any_class() {
        for d in LOCAL {
            for c in [
                DataClass::ClientConfidential,
                DataClass::UserContent,
                DataClass::NonSensitive,
            ] {
                assert_eq!(decide(c, d), Decision::Allowed, "{c:?} -> {d:?}");
            }
        }
    }

    #[test]
    fn confidential_to_external_is_denied() {
        for d in NON_LOCAL {
            assert_eq!(
                decide(DataClass::ClientConfidential, d),
                Decision::Denied(DenyReason::ConfidentialToExternal),
                "ClientConfidential -> {d:?}"
            );
        }
    }

    #[test]
    fn user_content_to_external_is_denied() {
        for d in NON_LOCAL {
            assert_eq!(
                decide(DataClass::UserContent, d),
                Decision::Denied(DenyReason::UserContentToExternal),
                "UserContent -> {d:?}"
            );
        }
    }

    #[test]
    fn non_sensitive_to_external_is_allowed() {
        for d in NON_LOCAL {
            assert_eq!(
                decide(DataClass::NonSensitive, d),
                Decision::Allowed,
                "{d:?}"
            );
        }
    }

    #[test]
    fn evaluate_is_deterministic() {
        let req = EgressRequest {
            data_class: DataClass::ClientConfidential,
            destination: Destination::RemoteModel,
        };
        let p = PrivacyPolicy;
        assert_eq!(p.evaluate(&req), p.evaluate(&req));
    }
}
