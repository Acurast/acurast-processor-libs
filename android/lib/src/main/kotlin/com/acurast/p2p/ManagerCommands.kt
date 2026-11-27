package com.acurast.p2p

/** A command of the processor's manager. */
sealed interface ManagerCommand {
    /** Asks for the processor's [ProcessorState]. */
    data object Status : ManagerCommand

    /** Restarts the phone; without [force] only while no deployment is running. */
    data class Reboot(val force: Boolean) : ManagerCommand

    /** Restarts the processor app; without [force] only while no deployment is running. */
    data class RestartApp(val force: Boolean) : ManagerCommand
}

/**
 * State of the processor, as reported to its manager.
 *
 * @property appVersion version of the processor app.
 * @property os operating system and its version.
 * @property batteryPercent battery level in percent, if known.
 * @property charging whether the phone is charging, if known.
 * @property busy whether a deployment is running or due.
 * @property uptimeSeconds seconds since the app started.
 */
data class ProcessorState(
    val appVersion: String,
    val os: String,
    val batteryPercent: Int?,
    val charging: Boolean?,
    val busy: Boolean,
    val uptimeSeconds: Long,
)

/** The processor's answer to a command of its manager. */
sealed interface ManagerCommandReply {
    data class Status(val state: ProcessorState) : ManagerCommandReply

    /** The command runs once the manager got the reply. */
    data object Accepted : ManagerCommandReply

    data class Refused(val reason: Refusal) : ManagerCommandReply

    enum class Refusal {
        /** The caller is not the manager of this processor. */
        Forbidden,

        /** The command is too old. */
        Stale,

        /** A deployment is running and the command was not forced. */
        Busy,

        /** The processor cannot run this command. */
        Unsupported,

        /** The command could not be run. */
        Failed,
    }
}

/**
 * Handles the commands of the processor's manager. The P2P library has already checked that they
 * come from the manager and are recent.
 */
interface ManagerCommandHandler {
    /** Account of this processor's manager (32 bytes), or `null` while it is not paired. */
    suspend fun manager(): ByteArray?

    /** Answer to [command]: the state, [ManagerCommandReply.Accepted] if it will run, or why not. */
    suspend fun prepare(command: ManagerCommand): ManagerCommandReply

    /** Runs a command answered with [ManagerCommandReply.Accepted]. */
    suspend fun execute(command: ManagerCommand)
}
