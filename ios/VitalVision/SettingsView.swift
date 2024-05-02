//
//  SettingsView.swift
//  VitalVision
//
//  Created by Emanuel Mairoll on 02.04.24.
//

import SwiftUI

struct SettingsView: View {
    @Binding var histSizeApi: Int
    @Binding var histSizeAnalytics: Int
    @Binding var maxInitialRttMs: Int
    @Binding var syncIntervalMin: Int
    @Binding var bleMacPrefix: String
    @Binding var maxSignalResolutionBit: Int
    @Binding var maxSignalSamplingRateHz: Int
    @Binding var enableMockDevices: Bool

    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section(header: Text("Device Settings")) {
                    SettingRow(label: "BLE Mac Prefix", placeholder: "AA:BB:CC:DD:EE:FF", value: $bleMacPrefix)
                    SettingRow(label: "Max Initial RTT (ms)", placeholder: "00", value: $maxInitialRttMs)
                    SettingRow(label: "Sync Interval (min)", placeholder: "00", value: $syncIntervalMin)
                }

                Section(header: Text("Signal Settings")) {
                    SettingRow(label: "History Size API", placeholder: "00", value: $histSizeApi)
                    SettingRow(label: "History Size Analytics", placeholder: "00", value: $histSizeAnalytics)
                    SettingRow(label: "Max Signal Resolution (Bit)", placeholder: "00", value: $maxSignalResolutionBit)
                    SettingRow(label: "Max Signal Sampling Rate (Hz)", placeholder: "00", value: $maxSignalSamplingRateHz)
                }

                Section(header: Text("Development")) {
                    Toggle("Enable Mock Devices", isOn: $enableMockDevices)
                }
            }
            .navigationTitle("Settings")
            /*
            .navigationBarItems(trailing: Button("Done") {
                dismiss()
            })*/
        }
    }
}

struct SettingRow<T>: View {
    let label: String
    let placeholder: String
    @Binding var value: T

    // For string values
    init(label: String, placeholder: String, value: Binding<T>) where T == String {
        self.label = label
        self.placeholder = placeholder
        self._value = value
    }
    
    init(label: String, placeholder: String, value: Binding<T>) where T == Int {
        self.label = label
        self.placeholder = placeholder
        self._value = value
    }

    var body: some View {
        HStack {
            Text(label)
            Spacer()
            
            if T.self == Int.self {
                TextField(placeholder, value: $value, formatter: NumberFormatter())
#if os(iOS)
                    .keyboardType(.numberPad)
#endif
                    .multilineTextAlignment(.trailing)
            } else if T.self == String.self {
                TextField(placeholder, text: $value as! Binding<String>)
                    .multilineTextAlignment(.trailing)
            } else {
                EmptyView()
            }
        }
    }
}


#Preview {
    SettingsView(histSizeApi: .constant(500), histSizeAnalytics: .constant(300), maxInitialRttMs: .constant(50), syncIntervalMin: .constant(1), bleMacPrefix: .constant("AA:BB:CC:DD:EE:FF"), maxSignalResolutionBit: .constant(16), maxSignalSamplingRateHz: .constant(30), enableMockDevices: .constant(false))

}
