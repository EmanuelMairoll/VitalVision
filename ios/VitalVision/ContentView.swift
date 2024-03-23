import SwiftUI

struct ContentView: View {
    
    @State var core:VitalVisionCore  = VitalVisionCore()
    @State var devices: [Device]? = nil

    
#if os(macOS)
    @State var selectedDevice: Device?
    
    var body: some View {
        NavigationSplitView {
            if let devices = devices {
                List(devices, id: \.uuid, selection: $selectedDevice) { device in
                    DevicePreviewView(core: core, device: device)
                        
                }
                .navigationTitle("BLE Devices")
            } else {
                Text("Loading...")
                ProgressView()
            }
        } detail: {
            if let device = selectedDevice {
                DeviceDetailView(core: core, device: device)
            } else {
                Text("No device selected")
            }
        }
        .onReceive(core.devicesSubject) { devices in
            self.devices = devices
        }
    }
#else
    var body: some View {
        NavigationStack {
            if let devices = devices {
                List(devices, id: \.uuid) { device in
                    DevicePreviewView(core: core, device: device)
                }
                .navigationTitle("BLE Devices")
            } else {
                Text("Loading...")
                ProgressView()
            }
        }
        .onReceive(core.devicesSubject) { devices in
            self.devices = devices
        }
    }
#endif

    
}





#Preview {
    ContentView()
}
