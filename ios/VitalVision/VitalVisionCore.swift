import Combine
import Foundation

class VitalVisionCore {

    public let devicesSubject: PassthroughSubject<[Device], Never>
    public let dataSubject: PassthroughSubject<(channelUuid: String, data: [Int32?]), Never>
        
    let notifications: NotificationService

    var appliedConfig: VvCoreConfig? = nil
    var vvcore: VvCore?

    init(){
        devicesSubject = PassthroughSubject<[Device], Never>()
        dataSubject = PassthroughSubject<(channelUuid: String, data: [Int32?]), Never>()
        
        notifications = NotificationService(devicesSubject: devicesSubject)
    }
    
    // not using VitalVisionCore as callback directly to break ARC cycle
    class Delegate: VvCoreDelegate {
        init(devicesSubject: PassthroughSubject<[Device], Never>, dataSubject: PassthroughSubject<(channelUuid: String, data: [Int32?]), Never>) {
            self.devicesSubject = devicesSubject
            self.dataSubject = dataSubject
        }
        
        public let devicesSubject: PassthroughSubject<[Device], Never>
        public let dataSubject: PassthroughSubject<(channelUuid: String, data: [Int32?]), Never>

        public weak var wself: VitalVisionCore?
        
        func devicesChanged(devices: [Device]) {
            Task {
                await MainActor.run {
                    devicesSubject.send(devices)
                }
            }
        }
        
        func newData(channelUuid: String, data: [Int32?]) {
            Task {
                await MainActor.run {
                    dataSubject.send((channelUuid: channelUuid, data: data))
                }
            }
        }
    }
    
    
    func applyConfig(config: AppConfig){
        let coreConfig = VvCoreConfig(
            histSizeApi: UInt32(config.histSizeApi),
            histSizeAnalytics: UInt32(config.histSizeAnalytics),
            maxInitialRttMs: UInt32(config.maxInitialRttMs),
            syncIntervalSec: UInt64(config.syncIntervalSec),
            enableMockDevices: config.enableMockDevices,
            analysisIntervalPoints: UInt32(config.analysisIntervalPoints),
            ecgAnalysisParams: EcgAnalysisParameters(
                samplingFrequency: config.ecgSamplingFrequency,
                filterCutoffLow: config.ecgFilterCutoffLow,
                filterOrder: UInt32(config.ecgFilterOrder),
                rPeakProminenceMadMultiple: config.ecgRPeakProminenceMadMultiple,
                rPeakDistance: UInt32(config.ecgRPeakDistance),
                rPeakPlateau: UInt32(config.ecgRPeakPlateau),
                hrMin: config.ecgHRRangeLow,
                hrMax: config.ecgHRRangeHigh,
                hrMaxDiff: config.ecgHRMaxDiff
            ),
            ppgAnalysisParams: PpgAnalysisParameters(
                samplingFrequency: config.ppgSamplingFrequency,
                filterCutoffLow: config.ppgFilterCutoffLow,
                filterCutoffHigh: config.ppgFilterCutoffHigh,
                filterOrder: UInt32(config.ppgFilterOrder),
                envelopeRange: UInt16(config.ppgEnvelopeRange),
                amplitudeMin: Double(config.ppgAmplitudeMin),
                amplitudeMax: Double(config.ppgAmplitudeMax),
                troughDepthMin: -0.25,
                troughDepthMax: 0.25,
                pulseWidthMin: 0.333,
                pulseWidthMax: 1.5 //TODO
            )
        )
        
        notifications.qualityThreshold = Float(config.notificationQualityThreshold)
        notifications.durationThreshold = TimeInterval(config.notificationDurationThresholdSec)
        notifications.watchedChannels = Set(config.additionalData.values.flatMap { $0.watchIds })
        
        // assume config has changed
        guard appliedConfig != coreConfig else {
            return
        }
        
        let delegate = Delegate(devicesSubject: devicesSubject, dataSubject: dataSubject)

        let vvcore = VvCore(config: coreConfig, delegate: delegate)
        vvcore.startBleLoop()

        // overwriting an old, non-nil vvcore (should) remove its last ARC reference
        self.vvcore = vvcore
        self.appliedConfig = coreConfig
        
        delegate.wself = self
    }
    
    func syncTime(){
        vvcore?.syncTime()
    }
    
    func pause(){
        vvcore?.pause()
    }
    
    func resume(){
        vvcore?.resume()
    }
}
