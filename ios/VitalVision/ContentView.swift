//
//  ContentView.swift
//  VitalVision
//
//  Created by Emanuel Mairoll on 19.03.24.
//

import SwiftUI

struct ContentView: View {
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Hello, world!")
            Text("2 + 2 = \(myRustyAdd(left: 2, right: 2))")
            
        }
        .padding()
            
    }
}

#Preview {
    ContentView()
}
