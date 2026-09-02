//! Typed data models for the Tailscale daemon state.
//!
//! These mirror the JSON produced by `tailscale status --json` (which is the
//! same shape as the LocalAPI `/localapi/v0/status` response). Only the fields
//! TailScout needs are modeled; everything else is ignored. Every field is
//! tolerant of being absent so we never fail to parse a slightly different
//! daemon version.

use serde::Deserialize;
use std::collections::HashMap;

/// Backend connection state reported by the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendState {
    /// Daemon needs the machine to log in.
    NeedsLogin,
    /// Logged in and connected.
    Running,
    /// Logged in but stopped (`tailscale down`).
    Stopped,
    /// Starting up.
    Starting,
    /// Anything we don't explicitly recognize.
    Other(String),
}

impl BackendState {
    fn from_str(value: &str) -> Self {
        match value {
            "NeedsLogin" => Self::NeedsLogin,
            "Running" => Self::Running,
            "Stopped" => Self::Stopped,
            "Starting" => Self::Starting,
            other => Self::Other(other.to_string()),
        }
    }

    /// True when the tailnet is up and traffic can flow.
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Human-friendly label for the header.
    pub fn label(&self) -> String {
        match self {
            Self::NeedsLogin => "Logged out".to_string(),
            Self::Running => "Connected".to_string(),
            Self::Stopped => "Disconnected".to_string(),
            Self::Starting => "Starting…".to_string(),
            Self::Other(value) if value.is_empty() => "Unknown".to_string(),
            Self::Other(value) => value.clone(),
        }
    }
}

/// A single node in the tailnet (either this machine or a peer).
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub public_key: String,
    pub host_name: String,
    pub dns_name: String,
    pub os: String,
    pub tailscale_ips: Vec<String>,
    pub allowed_ips: Vec<String>,
    pub cur_addr: String,
    pub relay: String,
    pub online: bool,
    pub exit_node: bool,
    pub exit_node_option: bool,
    pub active: bool,
    pub taildrop_target: i64,
    pub no_file_sharing_reason: String,
    pub user_id: u64,
    pub key_expiry: String,
    pub last_seen: String,
    pub last_handshake: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

impl Node {
    /// The primary (IPv4) Tailscale address, falling back to the first address.
    pub fn primary_ip(&self) -> Option<&str> {
        self.tailscale_ips
            .iter()
            .find(|ip| ip.contains('.'))
            .or_else(|| self.tailscale_ips.first())
            .map(String::as_str)
    }

    /// A display name: the hostname, falling back to the DNS name, then IP.
    pub fn display_name(&self) -> String {
        if !self.host_name.is_empty() {
            return self.host_name.clone();
        }
        if !self.dns_name.is_empty() {
            return self.dns_name.trim_end_matches('.').to_string();
        }
        self.primary_ip().unwrap_or("unknown").to_string()
    }

    /// DNS name without the trailing dot.
    pub fn clean_dns_name(&self) -> String {
        self.dns_name.trim_end_matches('.').to_string()
    }

    /// Stable target accepted by mutating CLI commands.
    pub fn cli_target(&self) -> Option<&str> {
        self.primary_ip().or_else(|| {
            let dns = self.dns_name.trim_end_matches('.');
            (!dns.is_empty()).then_some(dns)
        })
    }

    pub fn can_receive_taildrop(&self) -> bool {
        self.online && self.taildrop_target > 0 && self.no_file_sharing_reason.is_empty()
    }

    pub fn is_subnet_router(&self) -> bool {
        self.allowed_ips
            .iter()
            .any(|ip| !ip.ends_with("/32") && !ip.ends_with("/128"))
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserProfile {
    pub id: u64,
    pub login_name: String,
    pub display_name: String,
}

impl UserProfile {
    pub fn display_label(&self) -> String {
        if !self.display_name.is_empty() {
            return self.display_name.clone();
        }
        if !self.login_name.is_empty() {
            return self.login_name.clone();
        }
        if self.id == 0 {
            "Unknown user".to_string()
        } else {
            self.id.to_string()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Tailnet {
    pub name: String,
    pub magic_dns_suffix: String,
    pub magic_dns_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    #[serde(default, deserialize_with = "empty_if_null")]
    pub id: String,
    #[serde(default, deserialize_with = "empty_if_null")]
    pub nickname: String,
    #[serde(default, deserialize_with = "empty_if_null")]
    pub tailnet: String,
    #[serde(default, deserialize_with = "empty_if_null")]
    pub account: String,
    #[serde(default, deserialize_with = "empty_if_null")]
    pub selected: bool,
}

impl Profile {
    pub fn parse_list(input: &str) -> Result<Vec<Self>, serde_json::Error> {
        let profiles: Vec<Self> = serde_json::from_str(input)?;
        Ok(profiles
            .into_iter()
            .filter(|profile| !profile.display_name().is_empty())
            .collect())
    }

    pub fn display_name(&self) -> String {
        if !self.nickname.is_empty() {
            return self.nickname.clone();
        }
        if !self.account.is_empty() {
            return self.account.clone();
        }
        if !self.tailnet.is_empty() {
            return self.tailnet.clone();
        }
        self.id.clone()
    }

    pub fn switch_key(&self) -> String {
        if !self.id.is_empty() {
            self.id.clone()
        } else {
            self.display_name()
        }
    }
}

/// Parsed, ready-to-use view of the daemon status.
#[derive(Debug, Clone)]
pub struct Status {
    pub version: String,
    pub client_version: String,
    pub backend_state: BackendState,
    pub tun: bool,
    pub magic_dns_suffix: String,
    pub current_tailnet: Option<Tailnet>,
    pub health: Vec<String>,
    pub this_node: Option<Node>,
    pub peers: Vec<Node>,
    pub users: HashMap<u64, UserProfile>,
}

impl Status {
    /// Parse a `tailscale status --json` document.
    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        let raw: RawStatus = serde_json::from_str(input)?;
        Ok(raw.into_status())
    }

    /// Peers sorted: online first, then by display name (case-insensitive).
    pub fn sorted_peers(&self) -> Vec<Node> {
        let mut peers = self.peers.clone();
        peers.sort_by_cached_key(|node| (!node.online, node.display_name().to_lowercase()));
        peers
    }

    pub fn owner_label(&self, node: &Node) -> Option<String> {
        self.users
            .get(&node.user_id)
            .map(UserProfile::display_label)
    }

    /// Whether a peer belongs to the same Tailscale user as this device.
    /// Older daemon responses omit user IDs, so an unknown ID stays permissive.
    pub fn has_same_owner(&self, node: &Node) -> bool {
        self.this_node.as_ref().map_or(true, |this| {
            this.user_id == 0 || node.user_id == 0 || this.user_id == node.user_id
        })
    }

    /// Taildrop is usable only when both daemon capability and ownership allow it.
    pub fn can_send_taildrop_to(&self, node: &Node) -> bool {
        node.can_receive_taildrop() && self.has_same_owner(node)
    }

    pub fn display_version(&self) -> &str {
        if self.version.is_empty() {
            &self.client_version
        } else {
            &self.version
        }
    }

    pub fn exit_node_options(&self) -> Vec<Node> {
        self.sorted_peers()
            .into_iter()
            .filter(|node| node.exit_node_option)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Raw serde structs — the wire shape. Converted into the clean models above.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawStatus {
    #[serde(default, deserialize_with = "empty_if_null", rename = "Version")]
    version: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "ClientVersion")]
    client_version: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "BackendState")]
    backend_state: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "TUN")]
    tun: bool,
    #[serde(default, deserialize_with = "empty_if_null", rename = "MagicDNSSuffix")]
    magic_dns_suffix: String,
    #[serde(default, rename = "CurrentTailnet")]
    current_tailnet: Option<RawTailnet>,
    #[serde(default, deserialize_with = "empty_if_null", rename = "Health")]
    health: Vec<String>,
    #[serde(default, rename = "Self")]
    self_node: Option<RawNode>,
    #[serde(default, deserialize_with = "empty_if_null", rename = "Peer")]
    peer: HashMap<String, RawNode>,
    #[serde(default, deserialize_with = "empty_if_null", rename = "User")]
    user: HashMap<String, RawUserProfile>,
}

impl RawStatus {
    fn into_status(self) -> Status {
        let users = self
            .user
            .into_iter()
            .filter_map(|(id, user)| id.parse::<u64>().ok().map(|id| (id, user.into_user(id))))
            .collect();

        Status {
            version: self.version,
            client_version: self.client_version,
            backend_state: BackendState::from_str(&self.backend_state),
            tun: self.tun,
            magic_dns_suffix: self.magic_dns_suffix,
            current_tailnet: self.current_tailnet.map(RawTailnet::into_tailnet),
            health: self.health,
            this_node: self.self_node.map(RawNode::into_node),
            peers: self.peer.into_values().map(RawNode::into_node).collect(),
            users,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawNode {
    #[serde(default, deserialize_with = "empty_if_null", rename = "ID")]
    id: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "PublicKey")]
    public_key: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "HostName")]
    host_name: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "DNSName")]
    dns_name: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "OS")]
    os: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "TailscaleIPs")]
    tailscale_ips: Vec<String>,
    #[serde(default, deserialize_with = "empty_if_null", rename = "AllowedIPs")]
    allowed_ips: Vec<String>,
    #[serde(default, deserialize_with = "empty_if_null", rename = "CurAddr")]
    cur_addr: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "Relay")]
    relay: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "Online")]
    online: bool,
    #[serde(default, deserialize_with = "empty_if_null", rename = "ExitNode")]
    exit_node: bool,
    #[serde(default, deserialize_with = "empty_if_null", rename = "ExitNodeOption")]
    exit_node_option: bool,
    #[serde(default, deserialize_with = "empty_if_null", rename = "Active")]
    active: bool,
    #[serde(default, deserialize_with = "empty_if_null", rename = "TaildropTarget")]
    taildrop_target: i64,
    #[serde(
        default,
        deserialize_with = "empty_if_null",
        rename = "NoFileSharingReason"
    )]
    no_file_sharing_reason: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "UserID")]
    user_id: u64,
    #[serde(default, deserialize_with = "empty_if_null", rename = "KeyExpiry")]
    key_expiry: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "LastSeen")]
    last_seen: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "LastHandshake")]
    last_handshake: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "RxBytes")]
    rx_bytes: u64,
    #[serde(default, deserialize_with = "empty_if_null", rename = "TxBytes")]
    tx_bytes: u64,
}

impl RawNode {
    fn into_node(self) -> Node {
        Node {
            id: self.id,
            public_key: self.public_key,
            host_name: self.host_name,
            dns_name: self.dns_name,
            os: self.os,
            tailscale_ips: self.tailscale_ips,
            allowed_ips: self.allowed_ips,
            cur_addr: self.cur_addr,
            relay: self.relay,
            online: self.online,
            exit_node: self.exit_node,
            exit_node_option: self.exit_node_option,
            active: self.active,
            taildrop_target: self.taildrop_target,
            no_file_sharing_reason: self.no_file_sharing_reason,
            user_id: self.user_id,
            key_expiry: self.key_expiry,
            last_seen: self.last_seen,
            last_handshake: self.last_handshake,
            rx_bytes: self.rx_bytes,
            tx_bytes: self.tx_bytes,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawUserProfile {
    #[serde(default, deserialize_with = "empty_if_null", rename = "LoginName")]
    login_name: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "DisplayName")]
    display_name: String,
}

impl RawUserProfile {
    fn into_user(self, id: u64) -> UserProfile {
        UserProfile {
            id,
            login_name: self.login_name,
            display_name: self.display_name,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawTailnet {
    #[serde(default, deserialize_with = "empty_if_null", rename = "Name")]
    name: String,
    #[serde(default, deserialize_with = "empty_if_null", rename = "MagicDNSSuffix")]
    magic_dns_suffix: String,
    #[serde(
        default,
        deserialize_with = "empty_if_null",
        rename = "MagicDNSEnabled"
    )]
    magic_dns_enabled: bool,
}

impl RawTailnet {
    fn into_tailnet(self) -> Tailnet {
        Tailnet {
            name: self.name,
            magic_dns_suffix: self.magic_dns_suffix,
            magic_dns_enabled: self.magic_dns_enabled,
        }
    }
}

fn empty_if_null<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
