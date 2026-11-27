package com.acurast.p2p

import kotlinx.coroutines.runBlocking
import uniffi.acurast_p2p_ffi.ChainClient
import uniffi.acurast_p2p_ffi.KeyStore
import uniffi.acurast_p2p_ffi.ManagerCommandRefusal
import uniffi.acurast_p2p_ffi.ManagerCommand as FfiManagerCommand
import uniffi.acurast_p2p_ffi.ManagerCommandReply as FfiManagerCommandReply
import uniffi.acurast_p2p_ffi.ProcessorState as FfiProcessorState
import uniffi.acurast_p2p_ffi.P2pException
import kotlin.coroutines.cancellation.CancellationException

/**
 * [P2pSeedStore] as the library's [KeyStore]. The seed store creates the seed itself, so the
 * library never has one to store.
 */
internal class FfiKeyStore(private val seedStore: P2pSeedStore) : KeyStore {
    // Called from a library thread, never from the main thread.
    override fun loadSeed(): ByteArray = platformCall { runBlocking { seedStore.seed() } }

    override fun storeSeed(seed: ByteArray) {
        throw P2pException.Platform("the seed store creates the seed on first use")
    }
}

/**
 * [P2pChain] as the library's [ChainClient]. The library cannot handle a failing [account], so
 * the caller resolves it beforehand.
 */
internal class FfiChainClient(
    private val chain: P2pChain,
    private val account: ByteArray,
) : ChainClient {
    override fun account(): ByteArray = account

    override suspend fun genesisHash(): ByteArray = platformCall { chain.genesisHash() }

    override suspend fun stateCall(method: String, args: ByteArray): ByteArray =
        platformCall { chain.stateCall(method, args) }

    override suspend fun registerEndpoint(id: ByteArray, proof: ByteArray) =
        platformCall { chain.registerEndpoint(id, proof) }
}

/**
 * Runs [block] and reports its failures as [P2pException.Platform], the only error the library
 * accepts from the app.
 */
internal inline fun <T> platformCall(block: () -> T): T =
    try {
        block()
    } catch (e: CancellationException) {
        throw e
    } catch (e: P2pException) {
        throw e
    } catch (e: Throwable) {
        throw P2pException.Platform(e.message ?: e::class.java.simpleName)
    }

/**
 * [ManagerCommandHandler] as the native handler. The library expects every call to return, so
 * failures become `null` (no manager) or a [ManagerCommandRefusal.FAILED] refusal.
 */
internal class FfiManagerCommandHandler(
    private val handler: ManagerCommandHandler,
) : uniffi.acurast_p2p_ffi.ManagerCommandHandler {
    override suspend fun manager(): ByteArray? = orNull { handler.manager() }

    override suspend fun prepare(command: FfiManagerCommand): FfiManagerCommandReply =
        orNull { handler.prepare(command.toManagerCommand()).toFfi() }
            ?: FfiManagerCommandReply.Refused(ManagerCommandRefusal.FAILED)

    override suspend fun execute(command: FfiManagerCommand) {
        orNull { handler.execute(command.toManagerCommand()) }
    }
}

internal fun FfiManagerCommand.toManagerCommand(): ManagerCommand = when (this) {
    FfiManagerCommand.Status -> ManagerCommand.Status
    is FfiManagerCommand.Reboot -> ManagerCommand.Reboot(force)
    is FfiManagerCommand.RestartApp -> ManagerCommand.RestartApp(force)
}

internal fun ManagerCommandReply.toFfi(): FfiManagerCommandReply = when (this) {
    is ManagerCommandReply.Status -> FfiManagerCommandReply.Status(
        FfiProcessorState(
            appVersion = state.appVersion,
            os = state.os,
            batteryPercent = state.batteryPercent?.coerceIn(0, 100)?.toUByte(),
            charging = state.charging,
            busy = state.busy,
            uptimeSecs = state.uptimeSeconds.coerceAtLeast(0).toULong(),
        )
    )
    ManagerCommandReply.Accepted -> FfiManagerCommandReply.Accepted
    is ManagerCommandReply.Refused -> FfiManagerCommandReply.Refused(
        when (reason) {
            ManagerCommandReply.Refusal.Forbidden -> ManagerCommandRefusal.FORBIDDEN
            ManagerCommandReply.Refusal.Stale -> ManagerCommandRefusal.STALE
            ManagerCommandReply.Refusal.Busy -> ManagerCommandRefusal.BUSY
            ManagerCommandReply.Refusal.Unsupported -> ManagerCommandRefusal.UNSUPPORTED
            ManagerCommandReply.Refusal.Failed -> ManagerCommandRefusal.FAILED
        }
    )
}

private inline fun <T> orNull(block: () -> T): T? =
    try {
        block()
    } catch (e: CancellationException) {
        throw e
    } catch (e: Throwable) {
        null
    }
