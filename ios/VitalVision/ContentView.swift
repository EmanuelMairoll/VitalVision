import SwiftUI

enum SortBy {
    case timeDiscovered
    case id
    case participant
    case location
}

struct ContentView: View {
    
    @Binding var appConfig: AppConfig
    
    @State var core:VitalVisionCore = VitalVisionCore()
    @State var devices: [Device]? = nil
    @State var sortBy = SortBy.timeDiscovered
    @State var paused = false
    
    var sortedDevices: [String:[Device]]? {
        guard let devices = devices else { return nil }
        
        return switch sortBy {
        case .timeDiscovered:
            ["": devices]
        case .id:
            ["": devices.sorted { $0.id > $1.id }]
        case .participant:
            Dictionary(grouping: devices) { appConfig.additionalData[$0.id]?.participant ?? "" }
                .mapValues { $0.sorted { $0.id > $1.id } }
                
        case .location:
            Dictionary(grouping: devices) { appConfig.additionalData[$0.id]?.location ?? "" }
                .mapValues { $0.sorted { $0.id > $1.id } }
        }
    }

    
#if os(macOS)
    @State var selectedDevice: Device?
    
    var body: some View {
        NavigationSplitView {
            VStack {
                if let sortedDevices = sortedDevices {
                    List(selection: $selectedDevice) {
                        ForEach(sortedDevices.keys.sorted(), id: \.self) { key in
                            Section(header: Text(key)) {
                                ForEach(sortedDevices[key]!, id: \.id) { device in
                                    DevicePreviewView(core: core, device: device, additionalData: Binding (
                                        get: { appConfig.additionalData[device.id] ?? AdditionalDeviceData() },
                                        set: { appConfig.additionalData[device.id] = $0 }
                                    ))
                                }
                            }
                        }
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
                    Menu {
                        Picker("Sort by", selection: $sortBy) {
                            Text("Time Discovered").tag(SortBy.timeDiscovered)
                            Text("ID").tag(SortBy.id)
                            Text("Participant").tag(SortBy.participant)
                            Text("Location").tag(SortBy.location)
                        }
                        .pickerStyle(.inline)
                    } label: {
                        Label("Sort by", systemImage: "list.bullet")
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
                if let sortedDevices = sortedDevices {
                    List {
                        ForEach(sortedDevices.keys.sorted(), id: \.self) { key in
                            Section {
                                ForEach(sortedDevices[key]!, id: \.id) { device in
                                    DevicePreviewView(core: core, device: device, additionalData: Binding (
                                        get: { appConfig.additionalData[device.id] ?? AdditionalDeviceData() },
                                        set: { appConfig.additionalData[device.id] = $0 }
                                    ))
                                }
                            } header: {
                                if !key.isEmpty {
                                    Text(key)
                                }
                            }
                        }
                    }
                    .navigationTitle("BLE Devices")
                    EmptyView()
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
                ToolbarItem(placement: .navigationBarLeading) {
                    Menu {
                        Picker("Sort by", selection: $sortBy) {
                            Text("Time Discovered").tag(SortBy.timeDiscovered)
                            Text("ID").tag(SortBy.id)
                            Text("Participant").tag(SortBy.participant)
                            Text("Location").tag(SortBy.location)
                        }
                    } label: {
                        Label("Sort by", systemImage: "list.bullet")
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button {
                        if paused {
                            core.resume()
                        } else {
                            core.pause()
                        }
                        paused.toggle()
                    } label: {
                        Image(systemName: paused ? "play.fill" : "pause.fill")
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
