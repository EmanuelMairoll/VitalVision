import SwiftUI


struct ContentView: View {
    
    @Binding var appConfig: AppConfig
    
    @State var core:VitalVisionCore = VitalVisionCore()
    @State var devices: [Device]? = nil
#if os(macOS)
    @State var selectedDevice: Device?
    
    var body: some View {
        NavigationSplitView {
            VStack {
                if let devices = devices {
                    List(devices, id: \.id, selection: $selectedDevice) { device in
                        DevicePreviewView(core: core, device: device, additionalData: .constant(AdditionalDeviceData()))
                        
                    }
                    .navigationTitle("BLE Devices")
                } else {
                    Text("Loading...")
                    ProgressView()
                }
            }
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    SettingsLink {
                        Label("Settings", systemImage: "gearshape.fill")
                    }
                }
                ToolbarItem(placement: .automatic) {
                    Button {
                        core.syncTime()
                    } label: {
                        Image(systemName: "clock.arrow.2.circlepath")
                    }
                }
            }
        } detail: {
            if let device = selectedDevice {
                DeviceDetailView(core: core, device: device, additionalData: Binding(
                    get: { appConfig.additionalData[device.id] ?? AdditionalDeviceData() },
                    set: { appConfig.additionalData[device.id] = $0 }
                ))
            } else {
                Text("No device selected")
            }
        }
        .onAppear {
            core.applyConfig(config: appConfig)
        }
        .onReceive(core.devicesSubject) { devices in
            self.devices = devices
        }
    }
#else
    @State var showSettings: Bool = false

    var body: some View {
        NavigationStack {
            VStack {
                if let devices = devices {
                    List(devices, id: \.id) { device in
                        
                        DevicePreviewView(core: core, device: device, additionalData: Binding(
                            get: { appConfig.additionalData[device.id] ?? AdditionalDeviceData() },
                            set: { appConfig.additionalData[device.id] = $0 }
                        ))
                    }
                    .navigationTitle("BLE Devices")
                } else {
                    Text("Loading...")
                    ProgressView()
                }
            }
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button {
                        showSettings = true
                    } label: {
                        Image(systemName: "gearshape.fill")
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button {
                        core.syncTime()
                    } label: {
                        Image(systemName: "clock.arrow.2.circlepath")
                    }
                }
            }
        }
        .onAppear {
            core.applyConfig(config: appConfig)
        }
        .onChange(of: showSettings) { value in
            if !showSettings {
                devices = nil
                core.applyConfig(config: appConfig)
            }
        }
        .onReceive(core.devicesSubject) { devices in
            self.devices = devices
        }
        .fullScreenCover(isPresented: $showSettings) {
            SettingsView(
                config: $appConfig
            )
        }
    }
#endif
    
}

#Preview {
    ContentView(appConfig: .constant(AppConfig()))
}
