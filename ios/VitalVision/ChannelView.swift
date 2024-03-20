import SwiftUI
import Charts

extension ChannelType: CustomStringConvertible {
    public var description: String {
        switch self {
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
    let core: VvCore
    let channel: Channel
    @StateObject var channelDataCollector: AsyncCollector<[UInt16]>
    
    init(core: VvCore, channel: Channel) {
        self.core = core
        self.channel = channel
        let collector = core.collectChannelData(channelUuid: channel.uuid)
        _channelDataCollector = StateObject(wrappedValue: collector)
    }
    
    var body: some View {
        VStack {
            Chart {
                if let values = channelDataCollector.data {
                    ForEach(values.indices, id: \.self) { index in
                        LineMark(
                            x: .value("Time", index),
                            y: .value("Value", values[index])
                        )
                    }
                }
            }
            .frame(height: 300)
            //.chartYScale(domain: [0, 60])
            Spacer()
            Text("\(channel.status)")
                .font(.title)
                .padding()

        }
        .navigationTitle(channel.name)
    }
}
