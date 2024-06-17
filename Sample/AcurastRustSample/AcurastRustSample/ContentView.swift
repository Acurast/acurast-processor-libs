//
//  ContentView.swift
//  AcurastRustSample
//
//  Created by Pablo Martinez Piles on 12/6/24.
//

import SwiftUI
import Acurastsdk

struct ContentView: View {
    let acurast = acurastRpc()
    
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            
            Text("Hello, world!")
        }
        .padding()
        .task {
            await acurast.getHeader(blockHash: nil, timeout: nil)?.parentHash
        }
    }
}

#Preview {
    ContentView()
}
