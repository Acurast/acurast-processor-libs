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
            let rpc = acurastRpc()
            let client = AcurastRpcClient(acurastRpc: rpc)
            let header = await client.getHeader(blockHash: nil, timeout: nil)
            print(header?.parentHash)
        }
    }
}

#Preview {
    ContentView()
}
