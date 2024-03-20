import Combine

class AsyncCollector<Value: Sendable>: ObservableObject {
    @Published var data: Value?
    
    private var task: Task<Void, Error>?
    
    init(_ asyncFunction: @escaping (Bool) async -> Value) {
        task = Task {
            let newValue = await asyncFunction(false)
            await MainActor.run {
                self.data = newValue
            }

            while true {
                let newValue = await asyncFunction(true)
                await MainActor.run {
                    self.data = newValue
                }
            }
        }
    }
}

extension VvCore {
    func collectDevices() -> AsyncCollector<[Device]> {
        AsyncCollector<[Device]> { wait in
            await self.devices(waitForChange: wait)
        }
    }
    
    func collectDevice(uuid: String) -> AsyncCollector<Device> {
        AsyncCollector<Device> { wait in
            await self.device(uuid: uuid, waitForChange: wait)
        }
    }
    func collectChannelData(channelUuid: String) -> AsyncCollector<[UInt16]> {
        AsyncCollector<[UInt16]> { wait in
            await self.channelData(channelUuid: channelUuid, waitForChange: wait)
        }
    }
}
