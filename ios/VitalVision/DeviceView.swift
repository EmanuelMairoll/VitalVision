import SwiftUI

extension DeviceStatus: CustomStringConvertible {
    public var description: String {
        switch self {
        case .ok:
            return "OK"
        case .signalIssue:
            return "Signal Issue"
        }
    }
}

struct DevicePreviewView: View {
    let core: VitalVisionCore
    let device: Device
    
    var body: some View {
        NavigationLink(destination: DeviceDetailView(core: core, device: device)) {
            HStack {
                VStack(alignment: .leading) {
                    Text("\(device.name) (\(device.serial))" )
                    Text(device.mac)
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
        .id(device.mac)
    }
}

struct DeviceDetailView: View {
    let core: VitalVisionCore
    let device: Device
    
    var body: some View {
        List {
            Section(header: Text("Device Info")) {
                Text("MAC: \(device.mac)")
                Text("Status: \(device.status.description)")
                Text("Connected: \(device.connected ? "YES" : "NO")")
                Text("Time Drift: \(device.driftUs / 1000)ms")
                Text("Battery Level: \(device.battery)%")
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
        .id(device.mac)
    }
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
