//
//  SettingsView.swift
//  VitalVision
//
//  Created by Emanuel Mairoll on 02.04.24.
//

import SwiftUI

struct SettingsView: View {
    @Binding var config: AppConfig
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section(header: Text("Time Settings")) {
                    SettingRow(label: "Maximum Initial RTT (ms)", placeholder: "00", value: $config.maxInitialRttMs)
                    SettingRow(label: "Sync Interval (sec)", placeholder: "00", value: $config.syncIntervalSec)
                }

                Section(header: Text("Historical Data")) {
                    SettingRow(label: "History Size API", placeholder: "00", value: $config.histSizeApi)
                    SettingRow(label: "History Size Analytics", placeholder: "00", value: $config.histSizeAnalytics)
                }

                Section(header: Text("Notification Settings")) {
                    SettingRow(label: "Notification Quality Threshold", placeholder: "0.0", value: $config.notificationQualityThreshold)
                    SettingRow(label: "Notification Duration Threshold (sec)", placeholder: "00", value: $config.notificationDurationThresholdSec)
                }

                Section(header: Text("Development Settings")) {
                    Toggle("Enable Mock Devices", isOn: $config.enableMockDevices)
                    NavigationLink(destination: AnalysisParametersView(config: $config)) {
                        Text("Configure Analysis Parameters")
                    }
                }
            }
            .navigationTitle("Settings")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct AnalysisParametersView: View {
    @Binding var config: AppConfig

    var body: some View {
        Form {
            Section(header: Text("PPG Settings")) {
                SettingRow(label: "PPG Sampling Frequency", placeholder: "00.0", value: $config.ppgSamplingFrequency)
                SettingRow(label: "PPG Filter Cutoff Low", placeholder: "00.0", value: $config.ppgFilterCutoffLow)
                SettingRow(label: "PPG Filter Cutoff High", placeholder: "00.0", value: $config.ppgFilterCutoffHigh)
                SettingRow(label: "PPG Filter Order", placeholder: "00", value: $config.ppgFilterOrder)
                SettingRow(label: "PPG Amplitude Min", placeholder: "00", value: $config.ppgAmplitudeMin)
                SettingRow(label: "PPG Amplitude Max", placeholder: "00", value: $config.ppgAmplitudeMax)
            }

            Section(header: Text("ECG Settings")) {
                SettingRow(label: "ECG Sampling Frequency", placeholder: "00.0", value: $config.ecgSamplingFrequency)
                SettingRow(label: "ECG Filter Cutoff Low", placeholder: "00.0", value: $config.ecgFilterCutoffLow)
                SettingRow(label: "ECG Filter Order", placeholder: "00", value: $config.ecgFilterOrder)
                SettingRow(label: "ECG R-Peak Prominence", placeholder: "00.0", value: $config.ecgRPeakProminenceMadMultiple)
                SettingRow(label: "ECG R-Peak Distance", placeholder: "00", value: $config.ecgRPeakDistance)
            }
        
        }
        .navigationTitle("Advanced Settings")
    }
}

struct SettingRow<T: Codable>: View {
    let label: String
    let placeholder: String
    @Binding var value: T

    var numberFormatter: NumberFormatter {
        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.allowsFloats = true
        return formatter
    }
    
    var body: some View {
        HStack {
            Text(label).foregroundColor(.primary)
            Spacer()
            TextField(placeholder, value: $value, formatter: numberFormatter)
                .multilineTextAlignment(.trailing)
#if os(iOS)
                .keyboardType(.decimalPad)
#endif
                .multilineTextAlignment(.trailing)
        }
    }
}


#Preview {
    SettingsView(config: .constant(AppConfig()))
}
