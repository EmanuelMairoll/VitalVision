import SwiftUI

struct ContentView: View {
    
    // DEFAULT SETTINGS VALUES
    @AppStorage("histSizeApi") var histSizeApi: Int = 500
    @AppStorage("histSizeAnalytics") var histSizeAnalytics: Int = 500
    @AppStorage("maxInitialRttMs") var maxInitialRttMs: Int = 1000
    @AppStorage("syncIntervalMin") var syncIntervalMin: Int = 1
    @AppStorage("bleMacPrefix") var bleMacPrefix: String = "AA:BB"
    @AppStorage("maxSignalResolutionBit") var maxSignalResolutionBit: Int = 8
    @AppStorage("maxSignalSamplingRateHz") var maxSignalSamplingRateHz: Int = 100
    @AppStorage("enableMockDevices") var enableMockDevices: Bool = false
    
    @State var core:VitalVisionCore = VitalVisionCore()
    @State var devices: [Device]? = nil
    @State var showSettings: Bool = false

#if os(macOS)
    @State var selectedDevice: Device?

    var body: some View {
        NavigationSplitView {
            VStack {
                if let devices = devices {
                    List(devices, id: \.mac, selection: $selectedDevice) { device in
                        DevicePreviewView(core: core, device: device)
                        
                    }
                    .navigationTitle("BLE Devices")
                } else {
                    Text("Loading...")
                    ProgressView()
                }
            }
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button {
                        showSettings = true
                    } label: {
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
                DeviceDetailView(core: core, device: device)
            } else {
                Text("No device selected")
            }
        }
        .onAppear {
            core.applyConfig(histSizeApi: histSizeApi, histSizeAnalytics: histSizeAnalytics, maxInitialRttMs: maxInitialRttMs, syncIntervalMin: syncIntervalMin, bleMacPrefix: bleMacPrefix, maxSignalResolutionBit: maxSignalResolutionBit, maxSignalSamplingRateHz: maxSignalSamplingRateHz, enableMockDevices: enableMockDevices)
        }
        .onReceive(core.devicesSubject) { devices in
            self.devices = devices
        }
        .sheet(isPresented: $showSettings) {
            SettingsView(
                histSizeApi: $histSizeApi,
                histSizeAnalytics: $histSizeAnalytics,
                maxInitialRttMs: $maxInitialRttMs,
                syncIntervalMin: $syncIntervalMin,
                bleMacPrefix: $bleMacPrefix,
                maxSignalResolutionBit: $maxSignalResolutionBit,
                maxSignalSamplingRateHz: $maxSignalSamplingRateHz,
                enableMockDevices: $enableMockDevices
            )
        }

    }
#else
    
    var body: some View {
        NavigationStack {
            VStack {
                if let devices = devices {
                    List(devices, id: \.mac) { device in
                        DevicePreviewView(core: core, device: device)
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
            applyConfig()
        }
        .onChange(of: showSettings) { value in
            if !showSettings {
                applyConfig()
            }
        }
        .onReceive(core.devicesSubject) { devices in
            self.devices = devices
        }
        .fullScreenCover(isPresented: $showSettings) {
            SettingsView(
                histSizeApi: $histSizeApi,
                histSizeAnalytics: $histSizeAnalytics,
                maxInitialRttMs: $maxInitialRttMs,
                syncIntervalMin: $syncIntervalMin,
                bleMacPrefix: $bleMacPrefix,
                maxSignalResolutionBit: $maxSignalResolutionBit,
                maxSignalSamplingRateHz: $maxSignalSamplingRateHz,
                enableMockDevices: $enableMockDevices
            )
        }
    }
#endif
    
    func applyConfig() {
        core.applyConfig(histSizeApi: histSizeApi, histSizeAnalytics: histSizeAnalytics, maxInitialRttMs: maxInitialRttMs, syncIntervalMin: syncIntervalMin, bleMacPrefix: bleMacPrefix, maxSignalResolutionBit: maxSignalResolutionBit, maxSignalSamplingRateHz: maxSignalSamplingRateHz, enableMockDevices: enableMockDevices)
    }
}

/*
#Preview {
    ContentView()
}*/
