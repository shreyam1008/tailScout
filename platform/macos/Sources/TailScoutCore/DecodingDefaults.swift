import Foundation

extension KeyedDecodingContainer {
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
