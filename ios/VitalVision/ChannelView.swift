import SwiftUI
import Charts

extension ChannelType: CustomStringConvertible {
    public var description: String {
        switch self {
        case .cnt:
            "CNT"
        case .ecg:
            "ECG"
        case .ppg:
            "PPG"
        }
    }
}

extension ChannelStatus: CustomStringConvertible {
    public var description: String {
        switch self {
        case .ok:
            return "OK"
        case .signalIssue:
            return "Signal Issue"
        }
    }
}

struct ChannelPreviewView: View {
    let channel: Channel
    
    var body: some View {
        VStack {
            Text(channel.name)
                .font(.title)
                .padding()
            Text(channel.channelType.description)
                .font(.title2)
                .padding()
            Spacer()
            Text(channel.status.description)
                .font(.title)
                .padding()
        }
        .navigationTitle(channel.name)
        .id(channel.uuid)
    }
}

struct ChannelDetailView: View {
    let core: VitalVisionCore
    let channel: Channel

    @State var channelData: [UInt16]? = nil
        
    var body: some View {
        VStack {
            Chart {
                if let values = channelData {
                    ForEach(values.indices, id: \.self) { index in
                        LineMark(
                            x: .value("Time", index),
                            y: .value("Value", values[index])
                        )
                    }
                }
            }
            .frame(height: 300)
            .chartYScale(domain: [UInt16.min, UInt16.max])
            Spacer()
            Text("\(channel.status)")
                .font(.title)
                .padding()

        }
        .navigationTitle(channel.name)
        .onReceive(core.dataSubject) { channelUuid, data in
            if channelUuid == channel.uuid {
                self.channelData = data
            }
        }
    }
}
