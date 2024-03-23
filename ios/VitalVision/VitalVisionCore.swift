//
//  Core.swift
//  VitalVision
//
//  Created by Emanuel Mairoll on 23.03.24.
//

import Combine

class VitalVisionCore: VvCoreDelegate {

    public let devicesSubject: PassthroughSubject<[Device], Never>
    public let dataSubject: PassthroughSubject<(channelUuid: String, data: [UInt16]), Never>
    
    // not using VitalVisionCore as callback directly to break ARC cycle
    class Delegate: VvCoreDelegate {
        public weak var wself: VitalVisionCore?
        
        func devicesChanged(devices: [Device]) {
            wself?.devicesChanged(devices: devices)
        }
        
        func newData(channelUuid: String, data: [UInt16]) {
            wself?.newData(channelUuid: channelUuid, data: data)

        }
    }
    
    public let vvcore: VvCore
    
    init() {
        let config = VvCoreConfig(histSize: 100, bleServiceFilter: "DCF31A27-A904-F3A3-AA4E-5AE42F1217B6", mockData: false)
        let delegate = Delegate()
        
        self.devicesSubject = PassthroughSubject<[Device], Never>()
        self.dataSubject = PassthroughSubject<(channelUuid: String, data: [UInt16]), Never>()
        
        vvcore = VvCore(config: config, delegate: delegate)
        vvcore.startBleLoop()
        
        delegate.wself = self
    }
    
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
