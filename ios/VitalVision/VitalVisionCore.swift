//
//  Core.swift
//  VitalVision
//
//  Created by Emanuel Mairoll on 23.03.24.
//

import Combine

class VitalVisionCore {

    public let devicesSubject: PassthroughSubject<[Device], Never>  = PassthroughSubject<[Device], Never>()
    public let dataSubject: PassthroughSubject<(channelUuid: String, data: [UInt16]), Never>  = PassthroughSubject<(channelUuid: String, data: [UInt16]), Never>()
     
    var appliedConfig: VvCoreConfig? = nil
    var vvcore: VvCore?

    // not using VitalVisionCore as callback directly to break ARC cycle
    class Delegate: VvCoreDelegate {
        init(devicesSubject: PassthroughSubject<[Device], Never>, dataSubject: PassthroughSubject<(channelUuid: String, data: [UInt16]), Never>) {
            self.devicesSubject = devicesSubject
            self.dataSubject = dataSubject
        }
        
        public let devicesSubject: PassthroughSubject<[Device], Never>
        public let dataSubject: PassthroughSubject<(channelUuid: String, data: [UInt16]), Never>

        public weak var wself: VitalVisionCore?
        
        func devicesChanged(devices: [Device]) {
            Task {
                await MainActor.run {
                    devicesSubject.send(devices)
                }
            }
        }
        
        func newData(channelUuid: String, data: [UInt16]) {
            Task {
                await MainActor.run {
                    dataSubject.send((channelUuid: channelUuid, data: data))
                }
            }
        }
    }
    
    
    func applyConfig(histSizeApi: Int, histSizeAnalytics: Int, maxInitialRttMs: Int, syncIntervalMin: Int, bleMacPrefix: String, maxSignalResolutionBit: Int, maxSignalSamplingRateHz: Int, enableMockDevices: Bool){
        let config = VvCoreConfig(histSizeApi: UInt32(histSizeApi), histSizeAnalytics: UInt32(histSizeAnalytics), maxInitialRttMs: UInt32(maxInitialRttMs), syncIntervalMin: UInt32(syncIntervalMin), bleMacPrefix: bleMacPrefix, maxSignalResolutionBit: UInt8(maxSignalResolutionBit), maxSignalSamplingRateHz: UInt8(maxSignalSamplingRateHz), enableMockDevices: enableMockDevices)
        
        // assume config has changed
        guard appliedConfig != config else {
            return
        }
        
        let delegate = Delegate(devicesSubject: devicesSubject, dataSubject: dataSubject)

        let vvcore = VvCore(config: config, delegate: delegate)
        vvcore.startBleLoop()

        // overwriting an old, non-nil vvcore (should) remove its last ARC reference
        self.vvcore = vvcore
        self.appliedConfig = config
        
        delegate.wself = self
        
    }
    
    func syncTime(){
        vvcore?.syncTime()
    }
}
