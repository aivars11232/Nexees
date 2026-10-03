package dev.nexees.feasibility

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent

object Notifications {
    const val RECEIVER_ID = 1
    private const val RECEIVER_CHANNEL = "receiver"
    private const val REQUEST_CHANNEL = "requests"

    fun channels(context: Context) {
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(
            NotificationChannel(RECEIVER_CHANNEL, "Receiver", NotificationManager.IMPORTANCE_LOW)
        )
        manager.createNotificationChannel(
            NotificationChannel(REQUEST_CHANNEL, "Requests that need you", NotificationManager.IMPORTANCE_HIGH)
        )
    }

    /** The visible notification that keeps the receiver running, with its stop control. */
    fun receiver(context: Context): Notification {
        val stop = PendingIntent.getService(
            context, 0,
            Intent(context, ReceiverService::class.java).setAction(ReceiverService.ACTION_STOP),
            PendingIntent.FLAG_IMMUTABLE,
        )
        return Notification.Builder(context, RECEIVER_CHANNEL)
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setContentTitle("Nexees receiver is running")
            .setContentText("The paired PC can send requests while this is shown.")
            .setOngoing(true)
            .addAction(Notification.Action.Builder(null, "Stop", stop).build())
            .build()
    }

    /** Asks the phone user to review a PC request Android would not let the app open by itself. */
    fun request(context: Context, command: String, app: String) {
        val review = PendingIntent.getActivity(
            context, command.hashCode(),
            Intent(context, MainActivity::class.java).putExtra(MainActivity.EXTRA_COMMAND, command),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val notification = Notification.Builder(context, REQUEST_CHANNEL)
            .setSmallIcon(android.R.drawable.stat_notify_more)
            .setContentTitle("PC asks to open $app")
            .setContentText("Tap to review the request.")
            .setContentIntent(review)
            .setAutoCancel(true)
            .build()
        context.getSystemService(NotificationManager::class.java).notify(command.hashCode(), notification)
    }

    /** After a reboot, when Android does not let the receiver start without the user. */
    fun resume(context: Context) {
        val open = PendingIntent.getActivity(
            context, 2, Intent(context, MainActivity::class.java), PendingIntent.FLAG_IMMUTABLE,
        )
        val notification = Notification.Builder(context, REQUEST_CHANNEL)
            .setSmallIcon(android.R.drawable.stat_notify_error)
            .setContentTitle("Nexees receiver is stopped")
            .setContentText("Open Nexees to start receiving PC requests again.")
            .setContentIntent(open)
            .setAutoCancel(true)
            .build()
        context.getSystemService(NotificationManager::class.java).notify(2, notification)
    }
}
