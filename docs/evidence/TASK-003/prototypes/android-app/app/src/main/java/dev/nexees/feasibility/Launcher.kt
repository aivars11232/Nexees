package dev.nexees.feasibility

import android.app.Activity
import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.provider.AlarmClock
import android.provider.Settings
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject

/**
 * Opens other apps for PC requests through public intents only. When Android
 * does not let the app start an activity from the background, the request
 * becomes a notification the phone user must act on; nothing taps, types or
 * bypasses the system on the user's behalf.
 */
object Launcher {
    private val targets: Map<String, Intent> = mapOf(
        "settings" to Intent(Settings.ACTION_SETTINGS),
        "alarms" to Intent(AlarmClock.ACTION_SHOW_ALARMS),
        "maps" to Intent(Intent.ACTION_VIEW, Uri.parse("geo:0,0?q=Riga")),
    )
    private val main = Handler(Looper.getMainLooper())

    /** What this phone can do for a PC request now, per action. */
    fun capabilities(context: Context): JSONArray {
        val list = JSONArray()
        for ((key, intent) in targets) {
            val installed = context.packageManager.resolveActivity(intent, PackageManager.MATCH_DEFAULT_ONLY) != null
            list.put(
                JSONObject()
                    .put("action", "launch_app:$key")
                    .put("availability", if (installed) "needs_user_action" else "unsupported")
                    .put(
                        "note",
                        if (installed) "opens at once only while Nexees is in the foreground; otherwise the phone user continues from a notification"
                        else "no installed app handles this public intent",
                    ),
            )
        }
        list.put(
            JSONObject().put("action", "screen_input").put("availability", "unsupported")
                .put("note", "tapping or typing into other apps is outside the baseline"),
        )
        return list
    }

    private fun update(context: Context, command: String, status: String, detail: String) {
        val reply = NativeCore.request(
            context, "command_update",
            JSONObject().put("id", command).put("status", status).put("detail", detail),
        )
        Log.i(TAG, "command $command -> $reply")
    }

    /** Called by the receiver service for each launch the core accepted. */
    fun handle(context: Context, command: String, app: String) {
        val target = targets[app]
        if (target == null) {
            update(context, command, "unsupported", "$app is not an app this phone opens")
            return
        }
        update(context, command, "running", "the phone's launcher took the request")
        if (MainActivity.visible) {
            context.startActivity(Intent(target).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
            update(context, command, "completed", "Android accepted the start of $app while Nexees was in the foreground")
            return
        }
        // The attempt: ask Android to bring Nexees forward from the background.
        context.startActivity(
            Intent(context, MainActivity::class.java)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                .putExtra(MainActivity.EXTRA_COMMAND, command),
        )
        main.postDelayed({
            if (!MainActivity.visible) {
                Thread {
                    Notifications.request(context, command, app)
                    update(
                        context, command, "needs_user_action",
                        "Android did not let Nexees open from the background; a notification asks the phone user to continue",
                    )
                }.start()
            }
        }, 2000)
    }

    /**
     * Runs only from the Continue button the phone user tapped. Expiry, lock
     * state and the grant are checked again now, not when the request arrived.
     */
    fun continueFromUser(activity: Activity, command: String): String {
        val state = NativeCore.request(activity, "command", JSONObject().put("id", command))
        val app = state.optString("action").removePrefix("launch_app:")
        val now = System.currentTimeMillis() / 1000
        val locked = activity.getSystemService(KeyguardManager::class.java).isKeyguardLocked
        val granted = NativeCore.request(activity, "grants").optBoolean("pc_to_phone")
        val (status, detail) = when {
            state.optString("status") !in setOf("running", "needs_user_action") ->
                return "nothing to continue: the request is ${state.optString("status")}"
            state.optLong("expires_at") <= now -> "expired" to "the phone user continued after the request expired; nothing was opened"
            locked -> return "the phone is locked; unlock it and continue again"
            !granted -> "denied" to "the pc_to_phone grant was revoked before the phone user continued"
            app !in targets -> "unsupported" to "$app is not an app this phone opens"
            else -> {
                activity.startActivity(Intent(targets.getValue(app)))
                "completed" to "opened $app after the phone user continued"
            }
        }
        update(activity, command, status, detail)
        return "$status: $detail"
    }

    fun decline(context: Context, command: String) =
        update(context, command, "denied", "the phone user declined the request")
}
