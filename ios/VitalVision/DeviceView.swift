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
                    Text(device.name)
                    Text(device.uuid)
                        .font(.caption)
                        .lineLimit(1)
                }
                Spacer()
                BatteryIndicator(level: device.battery)
                StatusIndicator(isOk: true /*device.isConnected*/)
            }
        }
        .id(device.uuid)
    }
}

struct DeviceDetailView: View {
    let core: VitalVisionCore
    let device: Device
        
    var body: some View {
        List {
            Section(header: Text("Device Info")) {
                Text("UUID: \(device.uuid)")
                Text("Status: \(device.status.description)")
                Text("Connected: \(device.connected ? "YES" : "NO")")
                Text("Battery Level: \(device.battery)%")
            }
            Section(header: Text("Channels")) {
                ForEach(device.channels, id: \.uuid) { channel in
                    NavigationLink(destination: ChannelDetailView(core: core, channel: channel)) {
                        
                        HStack {
                            VStack(alignment: .leading) {
                                Text(channel.name)
                                Text(channel.uuid)
                                    .font(.caption)
                                    .lineLimit(1)
                            }
                            Spacer()
                            Text(channel.channelType.description)
                            StatusIndicator(isOk: channel.status == .ok)
                        }
                    }
                }
            }
        }
        .navigationTitle(device.name)
        .id(device.uuid)
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
