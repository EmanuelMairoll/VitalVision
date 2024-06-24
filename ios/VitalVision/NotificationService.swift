//
//  NotificationService.swift
//  VitalVision
//
//  Created by Emanuel Mairoll on 27.05.24.
//

import Foundation
import Combine
import UserNotifications

class NotificationService {
    var devicesSubscription: AnyCancellable?

    public var qualityThreshold: Float? = nil
    public var durationThreshold: TimeInterval? = nil
    public var watchedChannels: Set<String> = []

    // Tracks channels and the time their signal quality fell below the threshold, as well as whether a notification has been sent for that channel
    private var trackedChannels: [String: (Date, Bool)] = [:]

    init(devicesSubject: PassthroughSubject<[Device], Never>) {
        devicesSubscription = devicesSubject.sink { [weak self] devices in
            self?.processDeviceUpdates(devices)
        }
        requestNotificationPermission()
    }

    private func requestNotificationPermission() {
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound, .badge]) { granted, error in
            if let error = error {
                print("Notification permission error: \(error)")
            }
        }
    }
    
    private func processDeviceUpdates(_ devices: [Device]) {
        let currentTime = Date()

        guard let qualityThreshold = qualityThreshold, let durationThreshold = durationThreshold else {
            return
        }
        
        for device in devices {
            for channel in device.channels {
                if !watchedChannels.contains(channel.id) {
                    trackedChannels.removeValue(forKey: channel.id)
                    continue
                }
                
                guard let signalQuality = channel.signalQuality, signalQuality < qualityThreshold else {
                    trackedChannels.removeValue(forKey: channel.id)
                    continue
                }
                
                if let (lowQualityTimeStart, notified) = trackedChannels[channel.id] {
                    if !notified && currentTime.timeIntervalSince(lowQualityTimeStart) > durationThreshold {
                        notify(device: device, channel: channel)
                        trackedChannels[channel.id] = (lowQualityTimeStart, true)
                    }
                } else {
                    trackedChannels[channel.id] = (currentTime, false)
                }
            }
        }
    }

    private func notify(device: Device, channel: Channel) {
        let content = UNMutableNotificationContent()
        content.title = "Low Signal Quality Alert"
        content.body = "\(device.name) - \(channel.name) has low signal quality."
        content.sound = UNNotificationSound.default

        let request = UNNotificationRequest(identifier: "\(device.id)-\(channel.id)", content: content, trigger: nil) // Trigger set to nil for immediate delivery
        UNUserNotificationCenter.current().add(request) { error in
            if let error = error {
                print("Error scheduling notification: \(error)")
            }
        }
    }
}

