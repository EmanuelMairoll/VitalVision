import SwiftUI

struct AdditionalDeviceData: Codable {
    var participant: String?
    var location: String?
    var other: String?
    
    var watchIds: [String] = []
}

struct DevicePreviewView: View {
    let core: VitalVisionCore
    let device: Device
    
    @Binding var additionalData: AdditionalDeviceData

    var body: some View {
        NavigationLink(destination: DeviceDetailView(core: core, device: device, additionalData: $additionalData)) {
            HStack {
                VStack(alignment: .leading) {
                    Text("\(device.name) (\(device.serial))" )
                    let grouped = Dictionary(grouping: device.channels, by: { $0.channelType.description })
                    HStack {
                        ForEach(grouped.keys.sorted(), id: \.self) { key in
                            Text(key)
                                .font(.caption)
                                .lineLimit(1)
                            ForEach(grouped[key]!, id: \.id) { channel in
                                StatusCircle(color: channel.qualityColor, size: 6.0)
                            }
                        }
                    }
                }
                Spacer()
                Text("\(device.driftUs / 1000)ms")
                    .font(.caption)
                BatteryIndicator(level: device.battery)
                StatusCircle(color: device.connected ? .green : .red)
            }
        }
    }
}


struct DeviceDetailView: View {
    let core: VitalVisionCore
    let device: Device
    
    @Binding var additionalData: AdditionalDeviceData
    
    var body: some View {
        List {
            Section(header: Text("Device Info")) {
                ListRow(key: "Serial", value: String(device.serial))
                ListRow(key: "Connected", value: device.connected ? "YES" : "NO")
                ListRow(key: "Sync Round Trip Delay", value: "\(device.driftUs / 1000)ms")
                ListRow(key: "Battery Level", value: "\(device.battery)%")
            }
            Section(header: Text("Additional Data")) {
                TextField("Participant", text: $additionalData.participant ?? "")
                TextField("Location", text: $additionalData.location ?? "")
                TextField("Other", text: $additionalData.other ?? "")
            }
            Section(header: Text("Channels")) {
                ForEach(device.channels, id: \.id) { channel in
                    #if os(macOS)
                    Text(channel.name)
                        .font(.title)
                    ChannelDetailView(core: core, channel: channel)
                    Divider()
                    #else
                    NavigationLink(destination: ChannelDetailView(core: core, channel: channel)) {
                        ChannelPreviewView(channel: channel, isWatched: additionalData.watchIds.contains(channel.id))
                    }
                    .contextMenu {
                        if additionalData.watchIds.contains(channel.id) {
                            Button(action: {
                                additionalData.watchIds.removeAll { $0 == channel.id }
                            }) {
                                Label("Unwatch", systemImage: "bell.slash")
                            }
                        } else {
                            Button(action: {
                                additionalData.watchIds.append(channel.id)
                            }) {
                                Label("Watch", systemImage: "bell")
                            }
                        }
                    }
                    #endif
                }
            }
        }
        .navigationTitle(device.name)
        .id(device.id)
    }
}

struct ListRow: View {
    let key: String
    let value: String
    
    var body: some View {
        HStack {
            Text(key)
            Spacer()
            Text(value)
        }
    }
}

func ??<T>(lhs: Binding<Optional<T>>, rhs: T) -> Binding<T> {
    Binding(
        get: { lhs.wrappedValue ?? rhs },
        set: { lhs.wrappedValue = $0 }
    )
}

struct StatusCircle: View {
    var color: Color
    var size: CGFloat = 10.0
    
    var body: some View {
        Circle()
            .fill(color)
            .frame(width: size, height: size)
    }
}

struct BatteryIndicator: View {
    var level: UInt8

    private var batteryImageName: String {
        switch level {
        case 75...255:
            return "battery.100"
        case 50..<75:
            return "battery.75"
        case 25..<50:
            return "battery.50"
        case 1..<25:
            return "battery.25"
        default:
            return "battery.0"
        }
    }
    
    var body: some View {
        VStack {
            Image(systemName: batteryImageName)
            Text("\(level)%")
                .font(.caption)

        }
    }
}
