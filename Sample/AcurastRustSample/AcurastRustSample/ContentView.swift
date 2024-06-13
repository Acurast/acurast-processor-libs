//
//  ContentView.swift
//  AcurastRustSample
//
//  Created by Pablo Martinez Piles on 12/6/24.
//

import SwiftUI

struct ContentView: View {
    let acurastRpc = AcurastRpc(noPointer: .init())
    
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            
            Text("Hello, world!")
        }
        .padding()
        .task {
            await acurastRpc.getHeader(blockHash: nil, timeout: nil)?.parentHash
        }
    }
}

#Preview {
    ContentView()
}
