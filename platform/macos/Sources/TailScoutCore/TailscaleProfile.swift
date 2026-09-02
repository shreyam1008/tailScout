import Foundation

public struct TailscaleProfile: Decodable, Equatable, Sendable {
    public let id: String
    public let nickname: String
    public let tailnet: String
    public let account: String
    public let selected: Bool

    enum CodingKeys: String, CodingKey {
        case id, nickname, tailnet, account, selected
    }

    public init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        id = try values.decodeString(forKey: .id)
        nickname = try values.decodeString(forKey: .nickname)
        tailnet = try values.decodeString(forKey: .tailnet)
        account = try values.decodeString(forKey: .account)
        selected = try values.decodeBool(forKey: .selected)
    }

    public static func parseList(_ input: String) throws -> [Self] {
        try JSONDecoder().decode([Self].self, from: Data(input.utf8))
            .filter { !$0.displayName.isEmpty }
    }

    public var displayName: String {
        [nickname, account, tailnet, id].first(where: { !$0.isEmpty }) ?? ""
    }

    public var switchKey: String {
        [id, nickname, account, tailnet].first(where: { !$0.isEmpty }) ?? ""
    }
}
