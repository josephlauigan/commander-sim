// Commander Practice: the iPad app around the Rust library. It starts practice mode's server on the iPad and shows
// the table in a web view, full screen, on true black.
import PracticeCore
import SwiftUI

@main
struct CommanderPracticeApp: App {
    @State private var url: URL? = nil
    @State private var failed = false

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
            .task { start() }
        }
    }

    /// the server reads the card data bundled with the app (Resources/data)
    private func start() {
        guard url == nil, let data = Bundle.main.resourceURL?.appendingPathComponent("data") else { return }
        let port = practice_start(data.path, 0)
        if port == 0 { failed = true } else { url = URL(string: "http://127.0.0.1:\(port)/") }
    }
}
