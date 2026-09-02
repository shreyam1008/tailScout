//! Integration tests for TailScout's pure backend: status parsing, the LocalAPI
//! HTTP response reader, and presentation helpers. No GUI required.

#[cfg(target_os = "linux")]
use tailscout::tailscale::localapi::parse_response;
use tailscout::tailscale::model::{BackendState, Profile, Status};
use tailscout::util::{human_bytes, os_label};

const SAMPLE_STATUS: &str = include_str!("../shared/fixtures/status.json");
const NULL_STATUS: &str = include_str!("../shared/fixtures/status-null.json");
const SAMPLE_PROFILES: &str = include_str!("../shared/fixtures/profiles.json");

#[test]
fn parses_top_level_fields() {
    let status = Status::from_json(SAMPLE_STATUS).expect("should parse");
    assert_eq!(status.version, "1.98.4-t9e69045b2");
    assert_eq!(status.client_version, "1.98.4");
    assert_eq!(status.display_version(), "1.98.4-t9e69045b2");
    assert!(status.tun);
    assert_eq!(status.backend_state, BackendState::Running);
    assert!(status.backend_state.is_running());
    assert_eq!(status.magic_dns_suffix, "tail9e520a.ts.net");
    assert_eq!(status.health, vec!["relay warning"]);
    let tailnet = status.current_tailnet.expect("tailnet present");
    assert_eq!(tailnet.name, "jkp.org.in");
    assert!(tailnet.magic_dns_enabled);
}

#[test]
fn parses_self_node() {
    let status = Status::from_json(SAMPLE_STATUS).unwrap();
    let me = status.this_node.expect("self present");
    assert_eq!(me.display_name(), "shre");
    assert_eq!(me.primary_ip(), Some("100.100.8.31")); // prefers IPv4
    assert_eq!(me.cli_target(), Some("100.100.8.31"));
    assert_eq!(me.clean_dns_name(), "shre.tail9e520a.ts.net");
    assert_eq!(me.user_id, 110841043178303);
}

#[test]
fn peers_sorted_online_first() {
    let status = Status::from_json(SAMPLE_STATUS).unwrap();
    assert_eq!(status.peers.len(), 3);

    let sorted = status.sorted_peers();
    assert_eq!(
        sorted
            .iter()
            .map(|node| node.display_name())
            .collect::<Vec<_>>(),
        ["guest-phone", "pixel", "dev-pc"]
    );
    assert!(sorted[0].online);
    assert!(!sorted[2].online);
    assert!(sorted[1].exit_node_option);
    assert!(sorted[1].can_receive_taildrop());
    assert!(sorted[2].is_subnet_router());
}

#[test]
fn resolves_owner_profiles() {
    let status = Status::from_json(SAMPLE_STATUS).unwrap();
    let phone = status
        .peers
        .iter()
        .find(|peer| peer.display_name() == "pixel")
        .unwrap();
    assert_eq!(
        status.owner_label(phone).as_deref(),
        Some("Shreyam Adhikari")
    );
    assert!(status.can_send_taildrop_to(phone));

    let guest = status
        .peers
        .iter()
        .find(|peer| peer.display_name() == "guest-phone")
        .unwrap();
    assert!(guest.can_receive_taildrop());
    assert!(!status.can_send_taildrop_to(guest));
}

#[test]
fn handles_empty_and_missing_fields() {
    // Minimal document: unknown state, no self, no peers.
    let status = Status::from_json(r#"{"BackendState":"NeedsLogin"}"#).unwrap();
    assert_eq!(status.backend_state, BackendState::NeedsLogin);
    assert_eq!(status.backend_state.label(), "Logged out");
    assert!(status.this_node.is_none());
    assert!(status.peers.is_empty());
    assert_eq!(status.version, "");
}

#[test]
fn handles_null_fields_from_tailscale() {
    let status = Status::from_json(NULL_STATUS).unwrap();
    assert_eq!(status.version, "");
    assert!(status.health.is_empty());
    assert!(status.peers.is_empty());
    let node = status.this_node.unwrap();
    assert_eq!(node.display_name(), "unknown");
    assert!(node.tailscale_ips.is_empty());
    assert!(!node.online);
}

#[test]
fn parses_shared_profiles() {
    let profiles = Profile::parse_list(SAMPLE_PROFILES).unwrap();
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0].display_name(), "Work");
    assert!(profiles[0].selected);
    assert_eq!(profiles[1].display_name(), "me@home.example");
    assert_eq!(profiles[1].switch_key(), "profile-b");
}

#[test]
fn unknown_backend_state_is_preserved() {
    let status = Status::from_json(r#"{"BackendState":"WeirdNewState"}"#).unwrap();
    assert_eq!(
        status.backend_state,
        BackendState::Other("WeirdNewState".to_string())
    );
    assert!(!status.backend_state.is_running());
}

#[test]
fn rejects_garbage_json() {
    assert!(Status::from_json("not json at all").is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn localapi_parses_content_length_body() {
    let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 17\r\n\r\n{\"Version\":\"1.0\"}";
    let body = parse_response(raw).expect("should parse");
    assert_eq!(body, "{\"Version\":\"1.0\"}");
}

#[cfg(target_os = "linux")]
#[test]
fn localapi_parses_chunked_body() {
    // 0x11 = 17 bytes for the JSON payload.
    let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n11\r\n{\"Version\":\"1.0\"}\r\n0\r\n\r\n";
    let body = parse_response(raw).expect("should parse chunked");
    assert_eq!(body, "{\"Version\":\"1.0\"}");
}

#[cfg(target_os = "linux")]
#[test]
fn localapi_rejects_non_2xx() {
    let raw = b"HTTP/1.1 403 Forbidden\r\nContent-Length: 7\r\n\r\ndenied!";
    assert!(parse_response(raw).is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn localapi_status_round_trips_into_model() {
    // A chunked status body should parse and then deserialize into Status.
    let body = r#"{"Version":"9.9","BackendState":"Running"}"#;
    let raw = format!(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{}\r\n0\r\n\r\n",
        body.len(),
        body
    );
    let parsed = parse_response(raw.as_bytes()).unwrap();
    let status = Status::from_json(&parsed).unwrap();
    assert_eq!(status.version, "9.9");
    assert!(status.backend_state.is_running());
}

#[test]
fn os_labels_are_friendly() {
    assert_eq!(os_label("linux"), "Linux");
    assert_eq!(os_label("windows"), "Windows");
    assert_eq!(os_label("android"), "Android");
    assert_eq!(os_label("macOS"), "macOS");
    assert_eq!(os_label("macos"), "macOS");
    assert_eq!(os_label("ios"), "iOS");
    assert_eq!(os_label(""), "Unknown");
    assert_eq!(os_label("plan9"), "Plan9");
}

#[test]
fn human_bytes_formats_sizes() {
    assert_eq!(human_bytes(0), "0 B");
    assert_eq!(human_bytes(512), "512 B");
    assert_eq!(human_bytes(1024), "1.0 KB");
    assert_eq!(human_bytes(1536), "1.5 KB");
    assert_eq!(human_bytes(1048576), "1.0 MB");
}
