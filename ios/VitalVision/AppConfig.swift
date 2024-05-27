//
//  AppConfig.swift
//  VitalVision
//
//  Created by Emanuel Mairoll on 26.05.24.
//

import SwiftUI

struct AppConfig {
    // Main Settings
    @AppStorage("histSizeApi") var histSizeApi: Int = 500
    @AppStorage("histSizeAnalytics") var histSizeAnalytics: Int = 500
    @AppStorage("maxInitialRttMs") var maxInitialRttMs: Int = 100
    @AppStorage("syncIntervalSec") var syncIntervalSec: Int = 60
    @AppStorage("analysisIntervalPoints") var analysisIntervalPoints: Int = 60
    @AppStorage("notificationQualityThreshold") var notificationQualityThreshold: Double = 0.5
    @AppStorage("notificationDurationThresholdSec") var notificationDurationThresholdSec: Int = 300
    @AppStorage("enableMockDevices") var enableMockDevices: Bool = false
    
    // Configurable via Device Detail View
    @AppStorage("additionalDeviceData") var additionalData: [String: AdditionalDeviceData] = [:]

    // Advanced settings
    @AppStorage("ppgSamplingFrequency") var ppgSamplingFrequency: Double = 30.0
    @AppStorage("ppgFilterCutoffLow") var ppgFilterCutoffLow: Double = 1.0
    @AppStorage("ppgFilterCutoffHigh") var ppgFilterCutoffHigh: Double = 10.0
    @AppStorage("ppgFilterOrder") var ppgFilterOrder: Int = 4
    @AppStorage("ppgEnvelopeRange") var ppgEnvelopeRange: Int = 23
    @AppStorage("ppgAmplitudeMin") var ppgAmplitudeMin: Int = 10
    @AppStorage("ppgAmplitudeMax") var ppgAmplitudeMax: Int = 2000

    @AppStorage("ecgSamplingFrequency") var ecgSamplingFrequency: Double = 30.0
    @AppStorage("ecgFilterCutoffLow") var ecgFilterCutoffLow: Double = 0.6
    @AppStorage("ecgFilterOrder") var ecgFilterOrder: Int = 1
    @AppStorage("ecgRPeakProminenceMadMultiple") var ecgRPeakProminenceMadMultiple: Double = 12.0
    @AppStorage("ecgRPeakDistance") var ecgRPeakDistance: Int = 10
    @AppStorage("ecgRPeakPlateau") var ecgRPeakPlateau: Int = 3
    @AppStorage("ecgHRRangeLow") var ecgHRRangeLow: Double = 40.0
    @AppStorage("ecgHRRangeHigh") var ecgHRRangeHigh: Double = 200.0
    @AppStorage("ecgHRMaxDiff") var ecgHRMaxDiff: Double = 20.0
}

extension Dictionary: RawRepresentable where Key: Codable, Value: Codable {

    public init?(rawValue: String) {
        guard
            let data = rawValue.data(using: .utf8),
            let result = try? JSONDecoder().decode([Key: Value].self, from: data)
        else { return nil }
        self = result
    }

    public var rawValue: String {
        guard
            let data = try? JSONEncoder().encode(self),
            let result = String(data: data, encoding: .utf8)
        else { return "{}" }
        return result
    }
}
