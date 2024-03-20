import SwiftUI

struct ContentView: View {
    
    @State var core = VvCore() // using @State to not reinstanciate core on every update
    
    @StateObject var devicesCollector: AsyncCollector<[Device]>

    init(){
        let core = VvCore()
        let collector = core.collectDevices()
        _devicesCollector = StateObject(wrappedValue: collector)
    }
    
#if os(macOS)
    @State var selectedDevice: Device?
    
    var body: some View {
        NavigationSplitView {
            if let devices = devicesCollector.data {
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
    }
#else
    var body: some View {
        NavigationStack {
            if let devices = devicesCollector.data {
                List(devices, id: \.uuid) { device in
                    DevicePreviewView(core: core, device: device)
                }
                .navigationTitle("BLE Devices")
            } else {
                Text("Loading...")
                ProgressView()
            }
        }
    }
#endif

}





#Preview {
    ContentView()
}
