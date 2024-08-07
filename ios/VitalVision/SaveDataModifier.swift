import SwiftUI
import Combine
import UniformTypeIdentifiers

struct SaveDataModifier: ViewModifier {
    @Binding var channelData: [Int32?]?
    let channelName: String
    
    @State private var showingSaveDialog = false
    @State private var filename: String = ""
    @State private var snapshot: [Int32?] = []

    @Environment(\.openURL) var openURL
    
    func body(content: Content) -> some View {
        content
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button(action: {
                        if let channelData = channelData {
                            self.snapshot = channelData
                            self.filename = "\(channelName) \(DateFormatter.localizedString(from: Date(), dateStyle: .medium, timeStyle: .short))"
                            showSaveDialog()
                        }
                    }) {
                        HStack {
                            Image(systemName: "square.and.arrow.down")
                            Text(channelName)
                        }
                    }
                    #if os(iOS)
                    .alert("Save Data", isPresented: $showingSaveDialog) {
                        TextField("Filename", text: $filename)
                                Button("OK", action: {
                                    let documentDirectory = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
                                    let fileURL = documentDirectory.appendingPathComponent("\(self.filename).bin")
                                    saveData(at: fileURL)
                                })
                                Button("Cancel", role: .cancel) { }
                            } message: {
                                Text("Enter a name for the data file.")
                            }
                    #endif
                }
            }
    }

    private func showSaveDialog() {
        #if os(macOS)
        let savePanel = NSSavePanel()
        savePanel.allowedContentTypes = [UTType(filenameExtension: "bin")!]
        savePanel.nameFieldStringValue = filename
        savePanel.begin { response in
            if response == .OK, let url = savePanel.url {
                saveData(at: url)
            }
        }
        #else
        self.showingSaveDialog = true
        #endif
    }

    private func saveData(at url: URL) {
        do {
            let data = snapshot.compactMap { $0 }.withUnsafeBytes {
                Data($0)
            }
            try data.write(to: url)
            print("Data saved to \(url.path)")
        } catch {
            print("Error saving data: \(error)")
        }
    }

    private var alert: Alert {
        Alert(
            title: Text("Save Data"),
            message: Text("Enter a name for the data file."),
            primaryButton: .default(Text("Save"), action: {
                let documentDirectory = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
                let fileURL = documentDirectory.appendingPathComponent("\(self.filename).bin")
                saveData(at: fileURL)
            }),
            secondaryButton: .cancel()
        )
    }
}
