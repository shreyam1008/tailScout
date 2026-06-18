import Foundation

public enum BackendState: Equatable, Sendable {
    case needsLogin
    case running
    case stopped
    case starting
    case other(String)

    public init(rawValue: String) {
        switch rawValue {
        case "NeedsLogin":
            self = .needsLogin
        case "Running":
            self = .running
        case "Stopped":
            self = .stopped
        case "Starting":
            self = .starting
        default:
            self = .other(rawValue)
        }
    }

    public var isRunning: Bool {
        self == .running
    }

    public var label: String {
        switch self {
        case .needsLogin:
            "Logged out"
        case .running:
            "Connected"
        case .stopped:
            "Disconnected"
        case .starting:
            "Starting..."
        case .other(let value):
            value.isEmpty ? "Unknown" : value
        }
    }
}

public struct Tailnet: Decodable, Equatable, Sendable {
    public let name: String
    public let magicDNSSuffix: String
    public let magicDNSEnabled: Bool

    enum CodingKeys: String, CodingKey {
        case name = "Name"
        case magicDNSSuffix = "MagicDNSSuffix"
        case magicDNSEnabled = "MagicDNSEnabled"
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        name = try container.decodeString(forKey: .name)
        magicDNSSuffix = try container.decodeString(forKey: .magicDNSSuffix)
        magicDNSEnabled = try container.decodeBool(forKey: .magicDNSEnabled)
    }
}

public struct UserProfile: Decodable, Equatable, Sendable {
    public var id: UInt64
    public let loginName: String
    public let displayName: String
    public let profilePicURL: String

    enum CodingKeys: String, CodingKey {
        case id = "ID"
        case loginName = "LoginName"
        case displayName = "DisplayName"
        case profilePicURL = "ProfilePicURL"
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decodeUInt64(forKey: .id)
        loginName = try container.decodeString(forKey: .loginName)
        displayName = try container.decodeString(forKey: .displayName)
        profilePicURL = try container.decodeString(forKey: .profilePicURL)
    }

    public var displayLabel: String {
        if !displayName.isEmpty {
            return displayName
        }
        if !loginName.isEmpty {
            return loginName
        }
        return id == 0 ? "Unknown user" : String(id)
    }
}

public struct TailscaleNode: Decodable, Equatable, Identifiable, Sendable {
    public let id: String
    public let publicKey: String
    public let hostName: String
    public let dnsName: String
    public let os: String
    public let tailscaleIPs: [String]
    public let allowedIPs: [String]
    public let addrs: [String]
    public let curAddr: String
    public let relay: String
    public let online: Bool
    public let exitNode: Bool
    public let exitNodeOption: Bool
    public let active: Bool
    public let taildropTarget: Int64
    public let noFileSharingReason: String
    public let userID: UInt64
    public let keyExpiry: String
    public let lastSeen: String
    public let lastHandshake: String
    public let rxBytes: UInt64
    public let txBytes: UInt64

    enum CodingKeys: String, CodingKey {
        case id = "ID"
        case publicKey = "PublicKey"
        case hostName = "HostName"
        case dnsName = "DNSName"
        case os = "OS"
        case tailscaleIPs = "TailscaleIPs"
        case allowedIPs = "AllowedIPs"
        case addrs = "Addrs"
        case curAddr = "CurAddr"
        case relay = "Relay"
        case online = "Online"
        case exitNode = "ExitNode"
        case exitNodeOption = "ExitNodeOption"
        case active = "Active"
        case taildropTarget = "TaildropTarget"
        case noFileSharingReason = "NoFileSharingReason"
        case userID = "UserID"
        case keyExpiry = "KeyExpiry"
        case lastSeen = "LastSeen"
        case lastHandshake = "LastHandshake"
        case rxBytes = "RxBytes"
        case txBytes = "TxBytes"
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decodeString(forKey: .id)
        publicKey = try container.decodeString(forKey: .publicKey)
        hostName = try container.decodeString(forKey: .hostName)
        dnsName = try container.decodeString(forKey: .dnsName)
        os = try container.decodeString(forKey: .os)
        tailscaleIPs = try container.decodeStringArray(forKey: .tailscaleIPs)
        allowedIPs = try container.decodeStringArray(forKey: .allowedIPs)
        addrs = try container.decodeStringArray(forKey: .addrs)
        curAddr = try container.decodeString(forKey: .curAddr)
        relay = try container.decodeString(forKey: .relay)
        online = try container.decodeBool(forKey: .online)
        exitNode = try container.decodeBool(forKey: .exitNode)
        exitNodeOption = try container.decodeBool(forKey: .exitNodeOption)
        active = try container.decodeBool(forKey: .active)
        taildropTarget = try container.decodeInt64(forKey: .taildropTarget)
        noFileSharingReason = try container.decodeString(forKey: .noFileSharingReason)
        userID = try container.decodeUInt64(forKey: .userID)
        keyExpiry = try container.decodeString(forKey: .keyExpiry)
        lastSeen = try container.decodeString(forKey: .lastSeen)
        lastHandshake = try container.decodeString(forKey: .lastHandshake)
        rxBytes = try container.decodeUInt64(forKey: .rxBytes)
        txBytes = try container.decodeUInt64(forKey: .txBytes)
    }

    public var stableKey: String {
        if !id.isEmpty {
            return id
        }
        if !publicKey.isEmpty {
            return publicKey
        }
        if let ip = primaryIP {
            return ip
        }
        return displayName
    }

    public var primaryIP: String? {
        tailscaleIPs.first(where: { $0.contains(".") }) ?? tailscaleIPs.first
    }

    public var cleanDNSName: String {
        dnsName.trimmingCharacters(in: CharacterSet(charactersIn: "."))
    }

    public var displayName: String {
        if !hostName.isEmpty {
            return hostName
        }
        if !cleanDNSName.isEmpty {
            return cleanDNSName
        }
        return primaryIP ?? "unknown"
    }

    public var cliTarget: String? {
        if let primaryIP {
            return primaryIP
        }
        if !cleanDNSName.isEmpty {
            return cleanDNSName
        }
        return nil
    }

    public var canReceiveTaildrop: Bool {
        online && taildropTarget > 0 && noFileSharingReason.isEmpty
    }

    public var isSubnetRouter: Bool {
        allowedIPs.contains { ip in
            !ip.hasSuffix("/32") && !ip.hasSuffix("/128")
        }
    }
}

public struct TailscaleStatus: Decodable, Equatable, Sendable {
    public let version: String
    public let clientVersion: String
    public let backendState: BackendState
    public let tun: Bool
    public let magicDNSSuffix: String
    public let currentTailnet: Tailnet?
    public let health: [String]
    public let thisNode: TailscaleNode?
    public let peers: [TailscaleNode]
    public let users: [UInt64: UserProfile]

    enum CodingKeys: String, CodingKey {
        case version = "Version"
        case clientVersion = "ClientVersion"
        case backendState = "BackendState"
        case tun = "TUN"
        case magicDNSSuffix = "MagicDNSSuffix"
        case currentTailnet = "CurrentTailnet"
        case health = "Health"
        case thisNode = "Self"
        case peer = "Peer"
        case user = "User"
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        version = try container.decodeString(forKey: .version)
        clientVersion = try container.decodeString(forKey: .clientVersion)
        backendState = BackendState(rawValue: try container.decodeString(forKey: .backendState))
        tun = try container.decodeBool(forKey: .tun)
        magicDNSSuffix = try container.decodeString(forKey: .magicDNSSuffix)
        currentTailnet = try container.decodeIfPresent(Tailnet.self, forKey: .currentTailnet)
        health = try container.decodeStringArray(forKey: .health)
        thisNode = try container.decodeIfPresent(TailscaleNode.self, forKey: .thisNode)

        let peerMap = try container.decodeIfPresent([String: TailscaleNode].self, forKey: .peer) ?? [:]
        peers = Array(peerMap.values)

        let rawUsers = try container.decodeIfPresent([String: UserProfile].self, forKey: .user) ?? [:]
        users = rawUsers.reduce(into: [UInt64: UserProfile]()) { result, entry in
            guard let id = UInt64(entry.key) else {
                return
            }
            var profile = entry.value
            profile.id = id
            result[id] = profile
        }
    }

    public static func parse(_ input: String) throws -> TailscaleStatus {
        try JSONDecoder().decode(TailscaleStatus.self, from: Data(input.utf8))
    }

    public static func parse(_ data: Data) throws -> TailscaleStatus {
        try JSONDecoder().decode(TailscaleStatus.self, from: data)
    }

    public var displayVersion: String {
        if !clientVersion.isEmpty {
            return clientVersion
        }
        return version
    }

    public var sortedPeers: [TailscaleNode] {
        peers.sorted { lhs, rhs in
            if lhs.online != rhs.online {
                return lhs.online && !rhs.online
            }
            return lhs.displayName.localizedCaseInsensitiveCompare(rhs.displayName) == .orderedAscending
        }
    }

    public var exitNodeOptions: [TailscaleNode] {
        sortedPeers.filter(\.exitNodeOption)
    }

    public func ownerLabel(for node: TailscaleNode) -> String? {
        users[node.userID]?.displayLabel
    }
}

public struct TailscaleProfile: Decodable, Equatable, Sendable {
    public let id: String
    public let nickname: String
    public let tailnet: String
    public let account: String
    public let selected: Bool

    enum CodingKeys: String, CodingKey {
        case id
        case nickname
        case tailnet
        case account
        case selected
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decodeString(forKey: .id)
        nickname = try container.decodeString(forKey: .nickname)
        tailnet = try container.decodeString(forKey: .tailnet)
        account = try container.decodeString(forKey: .account)
        selected = try container.decodeBool(forKey: .selected)
    }

    public static func parseList(_ input: String) throws -> [TailscaleProfile] {
        try JSONDecoder().decode([TailscaleProfile].self, from: Data(input.utf8))
    }

    public var displayName: String {
        if !nickname.isEmpty {
            return nickname
        }
        if !account.isEmpty {
            return account
        }
        if !tailnet.isEmpty {
            return tailnet
        }
        return id.isEmpty ? "Unnamed profile" : id
    }

    public var detail: String {
        [account, tailnet]
            .filter { !$0.isEmpty && $0 != displayName }
            .joined(separator: " - ")
    }

    public var switchKey: String {
        if !id.isEmpty {
            return id
        }
        if !nickname.isEmpty {
            return nickname
        }
        if !account.isEmpty {
            return account
        }
        return tailnet
    }
}

private extension KeyedDecodingContainer {
    func decodeString(forKey key: Key) throws -> String {
        try decodeIfPresent(String.self, forKey: key) ?? ""
    }

    func decodeStringArray(forKey key: Key) throws -> [String] {
        try decodeIfPresent([String].self, forKey: key) ?? []
    }

    func decodeBool(forKey key: Key) throws -> Bool {
        try decodeIfPresent(Bool.self, forKey: key) ?? false
    }

    func decodeInt64(forKey key: Key) throws -> Int64 {
        try decodeIfPresent(Int64.self, forKey: key) ?? 0
    }

    func decodeUInt64(forKey key: Key) throws -> UInt64 {
        try decodeIfPresent(UInt64.self, forKey: key) ?? 0
    }
}
