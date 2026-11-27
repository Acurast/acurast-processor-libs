package com.acurast.p2p

import android.content.Context
import uniffi.acurast_p2p_ffi.AssignmentListener
import uniffi.acurast_p2p_ffi.runtimeSupport
import uniffi.acurast_p2p_ffi.P2pNode as FfiP2pNode

/**
 * The processor's running P2P endpoint, registered on chain.
 */
interface P2pNode {
    /** The endpoint's ID, hex-encoded. */
    val endpointId: String

    /** Whether this start registered the endpoint on chain. */
    val registeredNow: Boolean

    /** URLs of the relays in use. */
    val relays: List<String>

    /**
     * Whether the endpoint is connected to its home relay right now: relay nodes can notify new
     * assignments only while it is.
     */
    val relayConnected: Boolean

    /**
     * Re-reads the relays listed on chain and returns whether they changed.
     */
    suspend fun refreshRelays(): Boolean

    /**
     * Starts answering the pings of peers registered on chain.
     */
    suspend fun servePing()

    /**
     * Pings the hex-encoded [endpointId], whose address comes from the registry.
     */
    suspend fun ping(endpointId: String): P2pPing

    /**
     * Starts accepting the notices of relay nodes that deployments were assigned to this
     * processor, and passes each to [onNotice].
     */
    suspend fun serveAssignments(onNotice: (P2pAssignmentNotice) -> Unit)

    /**
     * Starts accepting the commands of this processor's manager and passes them to [handler].
     */
    suspend fun serveManagerCommands(handler: ManagerCommandHandler)

    /**
     * Starts sending a network report with [appVersion] to the relay node of the home relay every
     * 30 minutes.
     */
    suspend fun startTelemetry(appVersion: String)

    /**
     * Closes the endpoint and its connections.
     */
    suspend fun shutdown()

    fun interface Factory {
        /**
         * Starts the endpoint, registering it on chain first if needed.
         */
        suspend fun start(): P2pNode
    }
}

/**
 * Outcome of [P2pNode.ping].
 *
 * @property connectMs milliseconds to set up the connection.
 * @property rttMs milliseconds from sending the ping to receiving the reply.
 * @property direct whether the connection went directly over IP rather than through a relay.
 * @property remoteAddr the peer's address on a direct connection.
 * @property local whether the direct connection stays in the local network.
 */
data class P2pPing(
    val connectMs: Long,
    val rttMs: Long,
    val direct: Boolean,
    val remoteAddr: String? = null,
    val local: Boolean = false,
)

/**
 * Notice from a relay node that deployments were assigned to this processor. Only a hint: the
 * assignments are read from the chain.
 *
 * @property block the block in which the deployments were assigned.
 * @property jobCount how many assignments the notice lists.
 */
data class P2pAssignmentNotice(val block: Long, val jobCount: Int)

/**
 * Whether a chain's runtime has the P2P layer.
 *
 * @property specVersion version of the runtime that was asked.
 * @property p2p whether the runtime offers the P2P layer; without it no endpoint can register.
 */
data class P2pRuntimeSupport(val specVersion: Long, val p2p: Boolean)

/**
 * Reads from the chain's runtime whether it has the P2P layer, before starting a node.
 */
suspend fun P2pChain.runtimeSupport(): P2pRuntimeSupport {
    // Reading the runtime version needs no account.
    val support = platformCall { runtimeSupport(FfiChainClient(this, account = ByteArray(0))) }
    return P2pRuntimeSupport(support.specVersion.toLong(), support.p2p)
}

/**
 * Creates a [P2pNode.Factory] that starts the node with the key from [seedStore] on [chain]. The
 * node reads the system's DNS servers through [context].
 */
fun P2pNodeFactory(context: Context, seedStore: P2pSeedStore, chain: P2pChain): P2pNode.Factory = P2pNode.Factory {
    AcurastP2pAndroid.init(context)
    val account = chain.account()
    val node = FfiP2pNode.start(FfiKeyStore(seedStore), FfiChainClient(chain, account))
    FfiBackedP2pNode(node)
}

private class FfiBackedP2pNode(private val node: FfiP2pNode) : P2pNode {
    override val endpointId: String = node.endpointId()
    override val registeredNow: Boolean = node.registeredNow()
    override val relays: List<String>
        get() = node.relays()
    override val relayConnected: Boolean
        get() = node.relayConnected()

    override suspend fun refreshRelays(): Boolean = node.refreshRelays()

    override suspend fun servePing() {
        node.servePing()
    }

    override suspend fun serveAssignments(onNotice: (P2pAssignmentNotice) -> Unit) {
        node.serveAssignments(object : AssignmentListener {
            override fun onAssignments(block: UInt, jobs: List<ByteArray>) {
                onNotice(P2pAssignmentNotice(block.toLong(), jobs.size))
            }
        })
    }

    override suspend fun serveManagerCommands(handler: ManagerCommandHandler) {
        node.serveManagerCommands(FfiManagerCommandHandler(handler))
    }

    override suspend fun startTelemetry(appVersion: String) {
        node.startTelemetry(appVersion)
    }

    override suspend fun ping(endpointId: String): P2pPing =
        node.ping(endpointId).let {
            P2pPing(it.connectMs.toLong(), it.rttMs.toLong(), it.direct, it.remoteAddr, it.local)
        }

    override suspend fun shutdown() {
        node.use { it.shutdown() }
    }
}
