package com.acurast.p2p

/**
 * Persistent storage of the seed of the processor's P2P key.
 */
fun interface P2pSeedStore {
    /**
     * Returns the stored 32-byte seed, creating and storing a new one on first use.
     */
    suspend fun seed(): ByteArray
}

/**
 * The chain, as reached by the processor.
 */
interface P2pChain {
    /**
     * The processor's account ID (32 bytes).
     */
    suspend fun account(): ByteArray

    /**
     * Hash of the chain's genesis block (32 bytes).
     */
    suspend fun genesisHash(): ByteArray

    /**
     * Calls the runtime API [method] with SCALE-encoded [args] at the best block and returns the
     * SCALE-encoded result.
     */
    suspend fun stateCall(method: String, args: ByteArray): ByteArray

    /**
     * Submits `AcurastP2p::register_endpoint(id, proof)` signed with the processor key and returns
     * once it is submitted.
     */
    suspend fun registerEndpoint(id: ByteArray, proof: ByteArray)
}
