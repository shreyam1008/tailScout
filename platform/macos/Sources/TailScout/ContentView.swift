import SwiftUI
import TailScoutCore
import UniformTypeIdentifiers

struct ContentView: View {
    @EnvironmentObject private var model: AppViewModel
    @State private var showingSendImporter = false
    @State private var showingReceiveImporter = false
    @State private var sendTarget: TailscaleNode?

    var body: some View {
        NavigationSplitView {
            SidebarView()
        } detail: {
            DetailView(
                receiveFiles: { showingReceiveImporter = true },
                sendFile: {
                    sendTarget = $0
                    showingSendImporter = true
                }
            )
        }
        .frame(minWidth: 980, minHeight: 640)
        .toolbar {
            ToolbarItemGroup {
                if model.isWorking { ProgressView().controlSize(.small) }
                Button {
                    Task { await model.refreshAll() }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .help("Refresh status and devices")
            }
        }
        .alert(item: $model.alert) {
            Alert(
                title: Text($0.title),
                message: Text($0.message),
                dismissButton: .default(Text("OK"))
            )
        }
        .sheet(item: $model.diagnosticResult) { DiagnosticView(result: $0) }
        .fileImporter(
            isPresented: $showingSendImporter,
            allowedContentTypes: [.item],
            allowsMultipleSelection: false,
            onCompletion: handleSendSelection
        )
        .fileImporter(
            isPresented: $showingReceiveImporter,
            allowedContentTypes: [.folder],
            allowsMultipleSelection: false,
            onCompletion: handleReceiveSelection
        )
    }

    private func handleSendSelection(_ selection: Result<[URL], Error>) {
        handle(selection) { url in
            guard let sendTarget else { return }
            Task { await model.sendFile(url, to: sendTarget) }
        }
    }

    private func handleReceiveSelection(_ selection: Result<[URL], Error>) {
        handle(selection) { url in Task { await model.receiveFiles(to: url) } }
    }

    private func handle(_ selection: Result<[URL], Error>, action: (URL) -> Void) {
        switch selection {
        case .success(let urls):
            if let url = urls.first { action(url) }
        case .failure(let error):
            model.presentImportError(error)
        }
    }
}
