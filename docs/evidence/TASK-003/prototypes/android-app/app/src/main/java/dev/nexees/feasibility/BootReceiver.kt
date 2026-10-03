package dev.nexees.feasibility

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

/**
 * Starts the receiver after a reboot only if the phone user turned that on.
 * If Android refuses the start, the user is told; nothing retries silently.
 */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_BOOT_COMPLETED) return
        val enabled = MainActivity.startAtBoot(context)
        Log.i(TAG, "boot completed; receiver at boot: $enabled")
        if (!enabled) return
        Notifications.channels(context)
        try {
            context.startForegroundService(Intent(context, ReceiverService::class.java))
            Log.i(TAG, "receiver start after boot requested")
        } catch (e: Exception) {
            Log.w(TAG, "Android refused to start the receiver after boot: $e")
            Notifications.resume(context)
        }
    }
}
