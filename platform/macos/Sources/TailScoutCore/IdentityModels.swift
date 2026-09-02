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
        let values = try decoder.container(keyedBy: CodingKeys.self)
        name = try values.decodeString(forKey: .name)
        magicDNSSuffix = try values.decodeString(forKey: .magicDNSSuffix)
        magicDNSEnabled = try values.decodeBool(forKey: .magicDNSEnabled)
    }
}

public struct UserProfile: Decodable, Equatable, Sendable {
    public var id: UInt64
    public let loginName: String
    public let displayName: String

    enum CodingKeys: String, CodingKey {
        case id = "ID"
        case loginName = "LoginName"
        case displayName = "DisplayName"
    }

    public init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        id = try values.decodeUInt64(forKey: .id)
        loginName = try values.decodeString(forKey: .loginName)
        displayName = try values.decodeString(forKey: .displayName)
    }

    public var displayLabel: String {
        [displayName, loginName, id == 0 ? "" : String(id)]
            .first(where: { !$0.isEmpty }) ?? "Unknown user"
    }
}
