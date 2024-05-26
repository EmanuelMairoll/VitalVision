import SwiftUI

@main
struct VitalVisionApp: App {
    @State var appConfig = AppConfig()
    
    var body: some Scene {
        WindowGroup {
            ContentView(appConfig: $appConfig)
        }
        #if os(macOS)
        Settings {
            SettingsView(
                config: $appConfig
            )
        }

        #endif
    }
}
