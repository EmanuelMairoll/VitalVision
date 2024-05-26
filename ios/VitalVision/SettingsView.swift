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
                Section(header: Text("Device Settings")) {
                    SettingRow(label: "Sync Interval (sec)", placeholder: "00", value: $config.syncIntervalSec)
                }

                Section(header: Text("Signal Settings")) {
                    SettingRow(label: "History Size API", placeholder: "00", value: $config.histSizeApi)
                    SettingRow(label: "History Size Analytics", placeholder: "00", value: $config.histSizeAnalytics)
                }

                Section(header: Text("Development")) {
                    Toggle("Enable Mock Devices", isOn: $config.enableMockDevices)
                }
                
                Section(header: Text("PPG Analysis")) {
                    
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


/*
#Preview {
    SettingsView(
        histSizeApi: .constant(100), enableMockDevices: .constant(false), syncIntervalMin: .constant(10), histSizeAnalytics: .constant(100))
    

}*/
