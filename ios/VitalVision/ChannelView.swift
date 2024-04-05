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
        HStack {
            VStack(alignment: .leading) {
                Text(channel.name)
                Text(channel.id)
                    .font(.caption)
                    .lineLimit(1)
            }
            Spacer()
            Text(channel.channelType.description)
            StatusIndicator(isOk: channel.status == .ok)
        }
    }
}

struct ChannelDetailView: View {
    let core: VitalVisionCore
    let channel: Channel

    @State var channelData: [UInt16?]? = nil
        
    var body: some View {
        VStack {
            Chart {
                if let values = channelData {
                    ForEach(values.indices, id: \.self) { index in
                        if let val = values[index] {
                            LineMark(
                                x: .value("Time", index),
                                y: .value("Value", val)
                            )
                        }
                    }
                }
            }
            .frame(height: 300)
            .labelsHidden()
            .chartYScale(domain: [channel.signalMin, channel.signalMax])
            Spacer()
            Text("\(channel.status)")
                .font(.title)
                .padding()

        }
        .navigationTitle(channel.name)
        .onReceive(core.dataSubject) { channelId, data in
            if channelId == channel.id {
                self.channelData = data
            }
        }
    }
}
