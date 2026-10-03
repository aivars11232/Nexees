package dev.nexees.feasibility

import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.util.Log
import org.json.JSONObject

/**
 * Keeps the receiver running only while its notification is shown, as a
 * connectedDevice foreground service started from a user action. The Rust
 * core owns the socket, TLS and dispatch; this service owns Android
 * lifecycle and hands accepted launch requests to [Launcher].
 */
class ReceiverService : Service() {
    private val main = Handler(Looper.getMainLooper())
    private var logged = 0
    private val poll = object : Runnable {
        override fun run() {
            Thread { pollOnce() }.start()
            main.postDelayed(this, 1000)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stopReceiving()
            return START_NOT_STICKY
        }
        Notifications.channels(this)
        startForeground(
            Notifications.RECEIVER_ID, Notifications.receiver(this),
            ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE,
        )
        // A null intent means Android restarted the service after the process died.
        Log.i(TAG, "receiver service started; restarted by Android: ${intent == null}")
        Thread {
            extractAssets(this)
            NativeCore.request(this, "capabilities_set", JSONObject().put("capabilities", Launcher.capabilities(this)))
            Log.i(TAG, "receiver_start ${NativeCore.request(this, "receiver_start", JSONObject().put("port", PORT))}")
        }.start()
        main.removeCallbacks(poll)
        main.post(poll)
        return START_STICKY
    }

    private fun pollOnce() {
        val pending = NativeCore.request(this, "pending_launches").optJSONArray("launches") ?: return
        for (i in 0 until pending.length()) {
            val launch = pending.getJSONObject(i)
            Launcher.handle(this, launch.getString("id"), launch.getString("app"))
        }
        val log = NativeCore.request(this, "receiver_status").optJSONArray("log") ?: return
        // The core keeps the last 64 lines; print the ones not yet printed.
        if (log.length() < logged) logged = 0
        for (i in logged until log.length()) Log.i(TAG, "receiver: ${log.getString(i)}")
        logged = log.length()
    }

    private fun stopReceiving() {
        main.removeCallbacks(poll)
        Log.i(TAG, "receiver_stop ${NativeCore.request(this, "receiver_stop")}")
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    override fun onDestroy() {
        main.removeCallbacks(poll)
        NativeCore.request(this, "receiver_stop")
        Log.i(TAG, "receiver service destroyed")
        super.onDestroy()
    }

    companion object {
        const val ACTION_STOP = "dev.nexees.feasibility.STOP"
        const val PORT = 47200
    }
}
