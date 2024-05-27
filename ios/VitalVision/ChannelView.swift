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

extension Channel {
    public var qualityDesc: String? {
        guard let quality = self.signalQuality else {
            return nil
        }
        switch quality {
        case 0..<0.5:
            return "Poor"
        case 0.5..<0.75:
            return "Fair"
        case 0.75...1:
            return "Good"
        default:
            return nil
        }
    }
    
    public var qualityColor: Color {
        guard let quality = self.signalQuality else {
            return .gray
        }
        switch quality {
        case 0..<0.5:
            return .red
        case 0.5..<0.75:
            return .yellow
        case 0.75...1:
            return .green
        default:
            return .gray
        }
    }
}

struct ChannelPreviewView: View {
    let channel: Channel
    let isWatched: Bool
    
    var body: some View {
        HStack {
            VStack(alignment: .leading) {
                Text(channel.name)
            }
            Spacer()
            if isWatched {
                Image(systemName: "bell")
                    .foregroundColor(.accentColor)
            }
            Text(channel.channelType.description)
            StatusCircle(color: channel.qualityColor)
        }
    }
}

struct ChannelDetailView: View {
    let core: VitalVisionCore
    let channel: Channel

    @State var channelData: [Int32?]? = nil

    // Computed property to get the maximum value of the last 50% of channelData
    var channelDataMax: Int32 {
        let halfIndex = (channelData?.count ?? 0) / 2
        return channelData?
            .suffix(from: halfIndex)
            .compactMap { $0 }
            .max() ?? 0
    }

    // Computed property to get the minimum value of the last 50% of channelData
    var channelDataMin: Int32 {
        let halfIndex = (channelData?.count ?? 0) / 2
        return channelData?
            .suffix(from: halfIndex)
            .compactMap { $0 }
            .min() ?? 0
    }

    // Computed property to calculate the range of the last 50% of channelData
    var channelDataRange: Int32 {
        return channelDataMax - channelDataMin
    }

    // Computed property to calculate the domain based on the range
    var channelDataDomain: some ScaleDomain {
        let rangeTwoThirds = Int64(Double(channelDataRange) * (2.0 / 3.0))
        let minVal = Int32(max(Int64(Int32.min), Int64(channelDataMin) - rangeTwoThirds))
        let maxVal = Int32(min(Int64(channelDataMax) + rangeTwoThirds, Int64(Int32.max)))
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
            .chartYScale(domain: channelDataDomain)
            .clipped()
            Spacer()
            if let quality = channel.qualityDesc {
                Text("Signal Quality: \(quality)")
                    .font(.title)
                    .padding()
            }
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
