import SwiftUI

struct SaveDataModifier: ViewModifier {
    @Binding var channelData: [UInt16?]?
    let channelName: String
    
    @State private var showingSaveDialog = false
    @State private var filename: String = ""
    @State private var snapshot: [UInt16?] = []
    
    func body(content: Content) -> some View {
        content
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: {
                        if let channelData = channelData {
                            self.snapshot = channelData
                            self.filename = "\(channelName) \(DateFormatter.localizedString(from: Date(), dateStyle: .medium, timeStyle: .short))"
                            self.showingSaveDialog = true
                        }
                    }) {
                        Image(systemName: "square.and.arrow.down")
                    }
                }
            }
            .alert("Save Data", isPresented: $showingSaveDialog) {
                TextField("Enter filename", text: $filename)
                Button("Save") {
                    self.saveData(name: self.filename)
                }
                Button("Cancel", role: .cancel) {}
            } message: {
                Text("Enter a name for the data file.")
            }
    }

    private func saveData(name: String) {
        let documentDirectory = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let fileURL = documentDirectory.appendingPathComponent("\(name).bin")

        do {
            let data = snapshot.compactMap { $0 }.withUnsafeBytes {
                Data($0)
            }
            try data.write(to: fileURL)
            print("Data saved to \(fileURL.path)")
        } catch {
            print("Error saving data: \(error)")
        }
    }

}
