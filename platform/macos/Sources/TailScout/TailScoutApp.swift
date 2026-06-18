import SwiftUI
import TailScoutCore

@main
struct TailScoutApp: App {
    @StateObject private var model = AppViewModel(client: TailscaleCLI())

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(model)
                .task {
                    await model.refreshAll()
                }
        }
        .commands {
            CommandGroup(after: .appInfo) {
                Button("Refresh Status") {
                    Task { await model.refreshAll() }
                }
                .keyboardShortcut("r", modifiers: [.command])
            }
        }
    }
}
