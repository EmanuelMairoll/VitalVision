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

    // Computed property to get the maximum value of the last 50% of channelData
    var channelDataMax: UInt16 {
        let halfIndex = (channelData?.count ?? 0) / 2
        return channelData?
            .suffix(from: halfIndex)
            .compactMap { $0 }
            .max() ?? 0
    }

    // Computed property to get the minimum value of the last 50% of channelData
    var channelDataMin: UInt16 {
        let halfIndex = (channelData?.count ?? 0) / 2
        return channelData?
            .suffix(from: halfIndex)
            .compactMap { $0 }
            .min() ?? 0
    }

    // Computed property to calculate the range of the last 50% of channelData
    var channelDataRange: UInt16 {
        return channelDataMax - channelDataMin
    }

    // Computed property to calculate the domain based on the range
    var channelDataDomain: some ScaleDomain {
        let rangeTwoThirds = Int64(Double(channelDataRange) * (2.0 / 3.0))
        let minVal: UInt16 = UInt16(max(0, Int64(channelDataMin) - rangeTwoThirds))
        let maxVal: UInt16 = UInt16(min(Int64(channelDataMax) + rangeTwoThirds, Int64(UInt16.max)))
        return [minVal, maxVal]
    }

    
    
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
            //.chartYScale(domain: [channel.signalMin, channel.signalMax])
            .chartYScale(domain: channelDataDomain)
            Spacer()
            Text("\(channel.status)")
                .font(.title)
                .padding()

        }
        .navigationTitle(channel.name)
        .modifier(SaveDataModifier(channelData: $channelData, channelName: channel.name))
        .onReceive(core.dataSubject) { channelId, data in
            if channelId == channel.id {
                self.channelData = data
            }
        }
    }

}
