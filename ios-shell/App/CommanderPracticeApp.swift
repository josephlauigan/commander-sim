// Commander Practice: the iPad app around the Rust library. It starts practice mode's server on the iPad and shows
// the table in a web view, full screen, on true black.
import PracticeCore
import SwiftUI

@main
struct CommanderPracticeApp: App {
    @State private var url: URL? = nil
    @State private var failed = false
    @Environment(\.scenePhase) private var phase

    var body: some Scene {
        WindowGroup {
            ZStack {
                Color.black.ignoresSafeArea()
                if let url {
                    WebView(url: url).ignoresSafeArea()
                } else if failed {
                    Text("The practice server didn't start.").foregroundStyle(.white)
                }
            }
            .preferredColorScheme(.dark)
            .statusBarHidden()
            .persistentSystemOverlays(.hidden)
            .task { start() }
            .onChange(of: phase) { _, now in if now == .active { Task { await revive() } } }
        }
    }

    /// The server reads the data bundled with the app (AppData), copied on first launch into Application Support,
    /// where saved games (and the autosave Continue uses) live too. If iOS closes the app, the next launch starts the
    /// server again and Continue finds the autosave. There is no gesture for the two-player option: the page hides it
    /// in the app, as the Briefcase app's did; practice_set_lan(true) is there for a future one.
    private func start() {
        guard url == nil,
              let bundle = Bundle.main.resourceURL,
              let support = try? FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask,
                                                          appropriateFor: nil, create: true)
        else { failed = true; return }
        let port = practice_start(bundle.appendingPathComponent("AppData").path, support.path, 0)
        if port == 0 { failed = true } else { url = URL(string: "http://127.0.0.1:\(port)/") }
    }

    /// Back in front after a while in the background: iOS may have closed the server's listening socket while the app
    /// was suspended. If the server no longer answers, start it again (on a new port); the page then offers Continue.
    private func revive() async {
        guard let url else { return }
        var req = URLRequest(url: url)
        req.timeoutInterval = 2
        if (try? await URLSession.shared.data(for: req)) != nil { return }
        practice_stop()
        self.url = nil
        start()
    }
}
