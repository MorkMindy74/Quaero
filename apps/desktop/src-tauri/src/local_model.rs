//! Shared local-only HTTP primitives for on-device model providers (#58).
//!
//! Extracted from the chat `ollama` module so the chat provider and the Evidence
//! provider use the **same** hardened, audited egress path — no drift between two
//! copies of the local-only enforcement. Desktop-only (network I/O); never in the
//! pure `quaero-core`.
//!
//! Guarantees, fail-closed and in code (not just config):
//! - only an `http` URL on a loopback host is accepted (no `https`, no userinfo,
//!   no remote/LAN host, rejects ambiguous `http://127.0.0.1@evil.com`);
//! - the reqwest client follows **no redirects** (a local server's 3xx `Location`
//!   can never re-send the body off-device) and ignores **ambient proxies**
//!   (`HTTP_PROXY`/`ALL_PROXY` never receive the body); no TLS stack is pulled in.

use std::time::Duration;

/// Hard cap on a local model's HTTP response body (bytes), read in a bounded loop
/// BEFORE any JSON parsing — a broken/untrusted local model cannot force the app
/// to buffer/deserialize an unbounded body (fail-closed resource guard). Shared by
/// the chat and Evidence providers so the whole Ollama boundary is symmetric.
pub const MAX_RESPONSE_BYTES: usize = 1_048_576; // 1 MiB

/// Read an HTTP response body in a bounded loop, failing closed if it exceeds
/// [`MAX_RESPONSE_BYTES`]. Never buffers/parses an unbounded body. The error is
/// generic (no content).
pub async fn read_bounded(mut resp: reqwest::Response) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = resp.chunk().await.map_err(map_send_error)? {
        if buf.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err("risposta del modello troppo grande (oltre il limite)".to_string());
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

/// Build the hardened local-only HTTP client shared by all on-device providers.
/// Redirects disabled + system/env proxies disabled + a request timeout.
pub fn build_local_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        // A local (possibly untrusted) server could answer 3xx with a remote
        // `Location`; reqwest's default policy would re-send the body off-device.
        // A local provider has no legitimate reason to follow a redirect → disable
        // them entirely; any 3xx then stays a non-success status handled as error.
        .redirect(reqwest::redirect::Policy::none())
        // reqwest's default client honours ambient HTTP_PROXY/ALL_PROXY; a proxy
        // would receive the body before the loopback endpoint is reached, even for
        // a 127.0.0.1 URL → disable system/env proxy detection entirely.
        .no_proxy()
        .build()
        .map_err(|e| format!("errore inizializzazione client locale: {e}"))
}

/// Enforce the local-only invariant **in code** (fail-closed). Accepts only an
/// `http` URL whose host is loopback (`127.0.0.1`, `localhost`, `::1`), with no
/// userinfo. Rejects remote hosts, `https`, credentials, and ambiguous URLs such
/// as `http://127.0.0.1@evil.com` (whose real host is `evil.com`).
pub fn validate_local_endpoint(raw: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(raw).map_err(|_| format!("endpoint non valido: {raw}"))?;
    if url.scheme() != "http" {
        return Err("endpoint non locale: è consentito solo http su loopback".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("endpoint non valido: credenziali nell'URL non consentite".to_string());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "endpoint non valido: host assente".to_string())?;
    if !is_loopback_host(host) {
        return Err(format!(
            "endpoint non locale: host {host} non è loopback (bloccato)"
        ));
    }
    Ok(())
}

/// Loopback iff `localhost` or an IP that `is_loopback()` (handles 127.0.0.0/8
/// and `::1`, with optional IPv6 brackets).
fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    let stripped = host.trim_start_matches('[').trim_end_matches(']');
    stripped
        .parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

/// Map a transport error to a friendly, local-only message (never leaks content).
pub fn map_send_error(e: reqwest::Error) -> String {
    if e.is_connect() {
        "modello locale non disponibile — avvia Ollama (http://127.0.0.1:11434)".to_string()
    } else if e.is_timeout() {
        "il modello locale non ha risposto entro il tempo limite".to_string()
    } else {
        format!("errore di comunicazione con il modello locale: {e}")
    }
}

/// Map a non-2xx status to a friendly message. Redirects are disabled, so any 3xx
/// is surfaced as a blocked redirect (the body never leaves loopback).
pub fn map_status_error(code: u16) -> String {
    match code {
        404 => "modello non trovato in Ollama (verifica il nome del modello)".to_string(),
        300..=399 => {
            "il modello locale ha tentato un redirect (bloccato: nessun dato esce da loopback)"
                .to_string()
        }
        other => format!("il modello locale ha risposto con stato {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_endpoints_are_accepted() {
        for ok in [
            "http://127.0.0.1:11434",
            "http://127.0.0.1",
            "http://localhost:11434",
            "http://localhost",
            "http://[::1]:11434",
        ] {
            assert!(validate_local_endpoint(ok).is_ok(), "should accept {ok}");
        }
    }

    #[test]
    fn non_local_endpoints_are_rejected_fail_closed() {
        for bad in [
            "http://remote.example:11434", // remote host
            "http://192.168.1.5:11434",    // LAN, not loopback
            "https://127.0.0.1:11434",     // https not allowed
            "http://user:pass@127.0.0.1",  // userinfo
            "http://127.0.0.1@evil.com",   // ambiguous: real host is evil.com
            "https://ollama.com/api",      // cloud
            "ftp://127.0.0.1",             // wrong scheme
            "not-a-url",                   // unparseable
        ] {
            assert!(validate_local_endpoint(bad).is_err(), "must reject {bad}");
        }
    }

    #[test]
    fn status_errors_are_friendly() {
        assert!(map_status_error(404).contains("modello non trovato"));
        assert!(map_status_error(500).contains("stato 500"));
        // Any 3xx is reported as a blocked redirect (never followed).
        assert!(map_status_error(307).contains("redirect"));
        assert!(map_status_error(308).contains("redirect"));
        assert!(map_status_error(301).contains("redirect"));
    }
}
