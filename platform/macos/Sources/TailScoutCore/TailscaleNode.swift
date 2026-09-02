import Foundation

public struct TailscaleNode: Decodable, Equatable, Identifiable, Sendable {
    public let id: String
    public let publicKey: String
    public let hostName: String
    public let dnsName: String
    public let os: String
    public let tailscaleIPs: [String]
    public let allowedIPs: [String]
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
        let values = try decoder.container(keyedBy: CodingKeys.self)
        id = try values.decodeString(forKey: .id)
        publicKey = try values.decodeString(forKey: .publicKey)
        hostName = try values.decodeString(forKey: .hostName)
        dnsName = try values.decodeString(forKey: .dnsName)
        os = try values.decodeString(forKey: .os)
        tailscaleIPs = try values.decodeStringArray(forKey: .tailscaleIPs)
        allowedIPs = try values.decodeStringArray(forKey: .allowedIPs)
        curAddr = try values.decodeString(forKey: .curAddr)
        relay = try values.decodeString(forKey: .relay)
        online = try values.decodeBool(forKey: .online)
        exitNode = try values.decodeBool(forKey: .exitNode)
        exitNodeOption = try values.decodeBool(forKey: .exitNodeOption)
        active = try values.decodeBool(forKey: .active)
        taildropTarget = try values.decodeInt64(forKey: .taildropTarget)
        noFileSharingReason = try values.decodeString(forKey: .noFileSharingReason)
        userID = try values.decodeUInt64(forKey: .userID)
        keyExpiry = try values.decodeString(forKey: .keyExpiry)
        lastSeen = try values.decodeString(forKey: .lastSeen)
        lastHandshake = try values.decodeString(forKey: .lastHandshake)
        rxBytes = try values.decodeUInt64(forKey: .rxBytes)
        txBytes = try values.decodeUInt64(forKey: .txBytes)
    }

    public var primaryIP: String? {
        tailscaleIPs.first(where: { $0.contains(".") }) ?? tailscaleIPs.first
    }

    public var cleanDNSName: String {
        dnsName.trimmingCharacters(in: CharacterSet(charactersIn: "."))
    }

    public var displayName: String {
        [hostName, cleanDNSName, primaryIP ?? ""].first(where: { !$0.isEmpty }) ?? "unknown"
    }

    public var osLabel: String {
        switch os.lowercased() {
        case "": "Unknown"
        case "linux": "Linux"
        case "windows": "Windows"
        case "macos", "darwin": "macOS"
        case "ios": "iOS"
        case "android": "Android"
        case "freebsd": "FreeBSD"
        default: os.prefix(1).uppercased() + os.dropFirst()
        }
    }

    public var stableKey: String {
        [id, publicKey, primaryIP ?? "", displayName].first(where: { !$0.isEmpty }) ?? "unknown"
    }

    public var cliTarget: String? {
        [primaryIP ?? "", cleanDNSName].first(where: { !$0.isEmpty })
    }

    public var canReceiveTaildrop: Bool {
        online && taildropTarget > 0 && noFileSharingReason.isEmpty
    }

    public var isSubnetRouter: Bool {
        allowedIPs.contains { !$0.hasSuffix("/32") && !$0.hasSuffix("/128") }
    }
}
