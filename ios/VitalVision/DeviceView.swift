import SwiftUI

struct AdditionalDeviceData: Codable {
    var participant: String?
    var location: String?
    var other: String?
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
                    Text(device.id)
                        .font(.caption)
                        .lineLimit(1)
                }
                Spacer()
                Text("\(device.driftUs / 1000)ms")
                    .font(.caption)
                BatteryIndicator(level: device.battery)
                StatusIndicator(isOk: device.connected)
            }
        }
        .id(device.id)
    }
}

struct DeviceDetailView: View {
    let core: VitalVisionCore
    let device: Device
    
    @Binding var additionalData: AdditionalDeviceData
    
    var body: some View {
        List {
            Section(header: Text("Device Info")) {
                Text("Serial: \(device.serial)")
                Text("Connected: \(device.connected ? "YES" : "NO")")
                Text("Time Drift: \(device.driftUs / 1000)ms")
                Text("Battery Level: \(device.battery)%")
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
                        ChannelPreviewView(channel: channel)
                    }
                    #endif
                }
            }
        }
        .navigationTitle(device.name)
        .id(device.id)
    }
}

func ??<T>(lhs: Binding<Optional<T>>, rhs: T) -> Binding<T> {
    Binding(
        get: { lhs.wrappedValue ?? rhs },
        set: { lhs.wrappedValue = $0 }
    )
}

struct StatusIndicator: View {
    var isOk: Bool
    
    var body: some View {
        Circle()
            .fill(isOk ? Color.green : Color.orange)
            .frame(width: 10, height: 10)
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
