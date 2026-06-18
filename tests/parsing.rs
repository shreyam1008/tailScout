//! Integration tests for TailScout's pure backend: status parsing, the LocalAPI
//! HTTP response reader, and presentation helpers. No GUI required.

#[cfg(unix)]
use tailscout::tailscale::localapi::parse_response;
use tailscout::tailscale::model::{BackendState, Status};
use tailscout::util::{human_bytes, os_label};

const SAMPLE_STATUS: &str = r#"
{
  "Version": "1.98.4-t9e69045b2",
  "ClientVersion": "1.98.4",
  "TUN": true,
  "BackendState": "Running",
  "MagicDNSSuffix": "tail9e520a.ts.net",
  "CurrentTailnet": {
    "Name": "jkp.org.in",
    "MagicDNSSuffix": "tail9e520a.ts.net",
    "MagicDNSEnabled": true
  },
  "Health": ["relay warning"],
  "User": {
    "110841043178303": {
      "ID": 110841043178303,
      "LoginName": "shreyama@jkp.org.in",
      "DisplayName": "Shreyam Adhikari",
      "ProfilePicURL": "https://example.invalid/photo.png"
    }
  },
  "Self": {
    "ID": "self-1",
    "HostName": "shre",
    "DNSName": "shre.tail9e520a.ts.net.",
    "OS": "linux",
    "TailscaleIPs": ["100.100.8.31", "fd7a:115c:a1e0::1"],
    "Online": true,
    "UserID": 110841043178303
  },
  "Peer": {
    "key1": {
      "ID": "peer-win",
      "HostName": "dev-pc",
      "DNSName": "dev-pc.tail9e520a.ts.net.",
      "OS": "windows",
      "TailscaleIPs": ["100.100.8.30"],
      "AllowedIPs": ["100.100.8.30/32", "10.10.0.0/16"],
      "Online": false,
      "RxBytes": 2048,
      "TxBytes": 1024
    },
    "key2": {
      "ID": "peer-phone",
      "HostName": "pixel",
      "DNSName": "pixel.tail9e520a.ts.net.",
      "OS": "android",
      "TailscaleIPs": ["100.100.8.32"],
      "Online": true,
      "ExitNodeOption": true,
      "TaildropTarget": 3,
      "UserID": 110841043178303
    }
  }
}
"#;

#[test]
fn parses_top_level_fields() {
    let status = Status::from_json(SAMPLE_STATUS).expect("should parse");
    assert_eq!(status.version, "1.98.4-t9e69045b2");
    assert_eq!(status.client_version, "1.98.4");
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
    assert_eq!(me.clean_dns_name(), "shre.tail9e520a.ts.net");
    assert_eq!(me.user_id, 110841043178303);
}

#[test]
fn peers_sorted_online_first() {
    let status = Status::from_json(SAMPLE_STATUS).unwrap();
    assert_eq!(status.peers.len(), 2);

    let sorted = status.sorted_peers();
    // Online "pixel" should come before offline "dev-pc".
    assert_eq!(sorted[0].display_name(), "pixel");
    assert!(sorted[0].online);
    assert_eq!(sorted[1].display_name(), "dev-pc");
    assert!(!sorted[1].online);
    assert!(!sorted[1].exit_node_option);
    assert!(sorted[0].exit_node_option);
    assert!(sorted[0].can_receive_taildrop());
    assert!(sorted[1].is_subnet_router());
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
    let status = Status::from_json(
        r#"{
            "Version": null,
            "ClientVersion": null,
            "BackendState": "Stopped",
            "MagicDNSSuffix": null,
            "Health": null,
            "Peer": null,
            "User": null,
            "Self": {
                "HostName": null,
                "TailscaleIPs": null,
                "AllowedIPs": null,
                "Online": null,
                "TaildropTarget": null
            }
        }"#,
    )
    .unwrap();
    assert_eq!(status.version, "");
    assert!(status.health.is_empty());
    assert!(status.peers.is_empty());
    let node = status.this_node.unwrap();
    assert_eq!(node.display_name(), "unknown");
    assert!(node.tailscale_ips.is_empty());
    assert!(!node.online);
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

#[cfg(unix)]
#[test]
fn localapi_parses_content_length_body() {
    let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 17\r\n\r\n{\"Version\":\"1.0\"}";
    let body = parse_response(raw).expect("should parse");
    assert_eq!(body, "{\"Version\":\"1.0\"}");
}

#[cfg(unix)]
#[test]
fn localapi_parses_chunked_body() {
    // 0x11 = 17 bytes for the JSON payload.
    let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n11\r\n{\"Version\":\"1.0\"}\r\n0\r\n\r\n";
    let body = parse_response(raw).expect("should parse chunked");
    assert_eq!(body, "{\"Version\":\"1.0\"}");
}

#[cfg(unix)]
#[test]
fn localapi_rejects_non_2xx() {
    let raw = b"HTTP/1.1 403 Forbidden\r\nContent-Length: 7\r\n\r\ndenied!";
    assert!(parse_response(raw).is_err());
}

#[cfg(unix)]
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
