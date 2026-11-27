package com.acurast.p2p

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.acurast_p2p_ffi.ManagerCommandRefusal
import uniffi.acurast_p2p_ffi.P2pException
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import uniffi.acurast_p2p_ffi.ManagerCommand as FfiManagerCommand
import uniffi.acurast_p2p_ffi.ManagerCommandReply as FfiManagerCommandReply

class FfiTest {

    @Test
    fun `loadSeed - returns the seed store's seed`() {
        val keyStore = FfiKeyStore { ByteArray(32) { 7 } }

        assertContentEquals(ByteArray(32) { 7 }, keyStore.loadSeed())
    }

    @Test
    fun `loadSeed - reports a failing seed store as a platform error`() {
        val keyStore = FfiKeyStore { error("keystore locked") }

        val error = assertFailsWith<P2pException.Platform> { keyStore.loadSeed() }
        assertEquals("keystore locked", error.reason)
    }

    @Test
    fun `storeSeed - is refused`() {
        val keyStore = FfiKeyStore { ByteArray(32) }

        assertFailsWith<P2pException.Platform> { keyStore.storeSeed(ByteArray(32)) }
    }

    @Test
    fun `stateCall - reports a failing chain as a platform error`() = runTest {
        val client = FfiChainClient(chain(stateCall = { throw IllegalStateException("rpc down") }), ByteArray(32))

        val error = assertFailsWith<P2pException.Platform> { client.stateCall("P2pApi_endpoints", ByteArray(0)) }
        assertEquals("rpc down", error.reason)
    }

    @Test
    fun `stateCall - lets cancellation through`() = runTest {
        val client = FfiChainClient(chain(stateCall = { throw CancellationException("stopped") }), ByteArray(32))

        assertFailsWith<CancellationException> { client.stateCall("P2pApi_endpoints", ByteArray(0)) }
    }

    @Test
    fun `account - returns the resolved account`() {
        val client = FfiChainClient(chain(), ByteArray(32) { 5 })

        assertContentEquals(ByteArray(32) { 5 }, client.account())
    }

    @Test
    fun `manager commands - pass commands and replies through`() = runTest {
        val ffi = FfiManagerCommandHandler(handler(prepare = { command ->
            when (command) {
                ManagerCommand.Status -> ManagerCommandReply.Status(STATE)
                else -> ManagerCommandReply.Refused(ManagerCommandReply.Refusal.Busy)
            }
        }))

        assertContentEquals(MANAGER, ffi.manager())
        assertEquals(
            FfiManagerCommandReply.Refused(ManagerCommandRefusal.BUSY),
            ffi.prepare(FfiManagerCommand.RestartApp(force = false)),
        )
        val status = ffi.prepare(FfiManagerCommand.Status) as FfiManagerCommandReply.Status
        assertEquals("1.2.3 (45)", status.state.appVersion)
        assertEquals(87.toUByte(), status.state.batteryPercent)
        assertEquals(3_600.toULong(), status.state.uptimeSecs)
    }

    @Test
    fun `manager commands - report failures as no manager and refusals`() = runTest {
        val ffi = FfiManagerCommandHandler(handler(
            manager = { error("chain unreachable") },
            prepare = { error("unknown") },
        ))

        assertNull(ffi.manager())
        assertEquals(
            FfiManagerCommandReply.Refused(ManagerCommandRefusal.FAILED),
            ffi.prepare(FfiManagerCommand.RestartApp(force = false)),
        )
    }

    private fun chain(
        stateCall: suspend () -> ByteArray = { ByteArray(0) },
    ): P2pChain = object : P2pChain {
        override suspend fun account(): ByteArray = ByteArray(32)
        override suspend fun genesisHash(): ByteArray = ByteArray(32)
        override suspend fun stateCall(method: String, args: ByteArray): ByteArray = stateCall()
        override suspend fun registerEndpoint(id: ByteArray, proof: ByteArray) = Unit
    }

    private fun handler(
        manager: suspend () -> ByteArray? = { MANAGER },
        prepare: suspend (ManagerCommand) -> ManagerCommandReply,
    ): ManagerCommandHandler = object : ManagerCommandHandler {
        override suspend fun manager(): ByteArray? = manager.invoke()
        override suspend fun prepare(command: ManagerCommand): ManagerCommandReply = prepare.invoke(command)
        override suspend fun execute(command: ManagerCommand) = Unit
    }

    companion object {
        private val MANAGER = ByteArray(32) { 7 }
        private val STATE = ProcessorState(
            appVersion = "1.2.3 (45)",
            os = "Android 16",
            batteryPercent = 87,
            charging = true,
            busy = false,
            uptimeSeconds = 3_600,
        )
    }
}
