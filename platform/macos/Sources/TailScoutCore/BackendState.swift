public enum BackendState: Equatable, Sendable {
    case needsLogin
    case running
    case stopped
    case starting
    case other(String)

    public init(rawValue: String) {
        self = switch rawValue {
        case "NeedsLogin": .needsLogin
        case "Running": .running
        case "Stopped": .stopped
        case "Starting": .starting
        default: .other(rawValue)
        }
    }

    public var isRunning: Bool { self == .running }

    public var label: String {
        switch self {
        case .needsLogin: "Logged out"
        case .running: "Connected"
        case .stopped: "Disconnected"
        case .starting: "Starting…"
        case .other(let value): value.isEmpty ? "Unknown" : value
        }
    }
}
