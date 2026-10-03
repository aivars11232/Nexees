package dev.nexees.feasibility

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Typeface
import android.os.Bundle
import android.text.Html
import android.util.Log
import android.view.WindowInsets
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import org.json.JSONObject

/**
 * Every grant, start and continue happens here, from a button the phone
 * user taps. Button labels are stable so the evidence harness can find them.
 */
class MainActivity : Activity() {
    private lateinit var status: TextView
    private lateinit var grant: Button
    private lateinit var boot: Button
    private lateinit var review: LinearLayout
    private lateinit var reviewText: TextView
    private lateinit var help: TextView
    private var command: String? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val column = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(32, 64, 32, 32)
        }
        fun button(label: String, action: () -> Unit) =
            Button(this).apply { text = label; setOnClickListener { action() } }.also { column.addView(it) }

        review = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; visibility = LinearLayout.GONE }
        reviewText = TextView(this).also { review.addView(it) }
        review.addView(Button(this).apply { text = "Continue"; setOnClickListener { decide(true) } })
        review.addView(Button(this).apply { text = "Decline"; setOnClickListener { decide(false) } })
        column.addView(review)

        button("Run probe") { runProbe() }
        grant = button("Allow PC requests") { toggleGrant() }
        button("Start receiver") { startForegroundService(Intent(this, ReceiverService::class.java)) }
        button("Stop receiver") { startService(Intent(this, ReceiverService::class.java).setAction(ReceiverService.ACTION_STOP)) }
        boot = button("Receiver at boot: off") { toggleBoot() }
        button("Ask PC to open HopToDesk") { askPc() }
        button("Open manual") { openManual() }
        status = TextView(this).apply { typeface = Typeface.MONOSPACE; textSize = 11f }.also { column.addView(it) }
        // Passive help: a TextView runs no script, follows no link and loads no image.
        help = TextView(this).also { column.addView(it) }
        // Android 15 and later draw apps edge to edge; keep the buttons clear of the system bars.
        val scroll = ScrollView(this).apply { addView(column) }
        scroll.setOnApplyWindowInsetsListener { view, insets ->
            val bars = insets.getInsets(WindowInsets.Type.systemBars())
            view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
            insets
        }
        setContentView(scroll)

        if (checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)
        }
        background("assets") {
            extractAssets(this)
            NativeCore.request(this, "grants")
        }
        showReview(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        showReview(intent)
    }

    override fun onResume() {
        super.onResume()
        visible = true
        refreshButtons()
    }

    override fun onPause() {
        visible = false
        super.onPause()
    }

    /** Shows a PC request for the user to continue or decline; opening this screen alone decides nothing. */
    private fun showReview(intent: Intent?) {
        command = intent?.getStringExtra(EXTRA_COMMAND) ?: return
        reviewText.text = "A PC request is waiting: $command"
        review.visibility = LinearLayout.VISIBLE
    }

    private fun decide(continueRequest: Boolean) {
        val id = command ?: return
        command = null
        review.visibility = LinearLayout.GONE
        background("review") {
            val result = if (continueRequest) {
                Launcher.continueFromUser(this, id)
            } else {
                Launcher.decline(this, id)
                "declined"
            }
            JSONObject().put("result", result)
        }
    }

    private fun runProbe() = background("probe") {
        extractAssets(this)
        NativeCore.request(this, "probe", JSONObject().put("name", "app-${System.currentTimeMillis()}"))
    }

    private fun toggleGrant() = background("grant") {
        val enabled = NativeCore.request(this, "grants").optBoolean("pc_to_phone")
        NativeCore.request(this, "grant", JSONObject().put("direction", "pc_to_phone").put("enabled", !enabled))
    }

    private fun toggleBoot() {
        val enabled = !startAtBoot(this)
        getSharedPreferences(PREFERENCES, MODE_PRIVATE).edit().putBoolean(START_AT_BOOT, enabled).commit()
        Log.i(TAG, "receiver at boot: $enabled")
        refreshButtons()
    }

    private fun askPc() = background("send") {
        NativeCore.request(
            this, "send",
            JSONObject().put("address", PC_ADDRESS)
                .put("action", JSONObject().put("kind", "launch_app").put("app", "hoptodesk")),
        )
    }

    private fun openManual() = background("manual") {
        val chapter = NativeCore.request(this, "chapters").getJSONArray("chapters").getString(0)
        val html = NativeCore.request(this, "help", JSONObject().put("chapter", chapter)).getString("html")
        runOnUiThread { help.text = Html.fromHtml(html, Html.FROM_HTML_MODE_COMPACT) }
        JSONObject().put("chapter", chapter).put("html_bytes", html.length)
    }

    private fun refreshButtons() = Thread {
        val enabled = NativeCore.request(this, "grants").optBoolean("pc_to_phone")
        runOnUiThread {
            grant.text = if (enabled) "Revoke PC requests" else "Allow PC requests"
            boot.text = if (startAtBoot(this)) "Receiver at boot: on" else "Receiver at boot: off"
        }
    }.start()

    /** Runs [work] off the main thread, logs its reply for the harness and shows it. */
    private fun background(label: String, work: () -> JSONObject) = Thread {
        val reply = try {
            work()
        } catch (e: Exception) {
            JSONObject().put("error", e.toString())
        }
        Log.i(TAG, "$label ${if (label == "probe") summary(reply) else reply.toString()}")
        runOnUiThread {
            status.text = "$label: ${summary(reply)}"
            refreshButtons()
        }
    }.start()

    private fun summary(reply: JSONObject): String {
        val checks = reply.optJSONArray("checks") ?: return reply.toString(2)
        val failed = (0 until checks.length()).map { checks.getJSONObject(it) }.filter { !it.getBoolean("passed") }
        return "passed=${reply.optBoolean("passed")} checks=${checks.length()} failed=${failed.map { it.getString("id") }} " +
            "target=${reply.optString("target")} sqlite=${reply.optString("sqlite")}"
    }

    companion object {
        const val EXTRA_COMMAND = "dev.nexees.feasibility.COMMAND"
        private const val PREFERENCES = "receiver"
        private const val START_AT_BOOT = "start_at_boot"
        /** The PC as the Android emulator sees the host's loopback interface. */
        private const val PC_ADDRESS = "10.0.2.2:47100"

        @Volatile
        var visible = false

        fun startAtBoot(context: Context): Boolean =
            context.getSharedPreferences(PREFERENCES, MODE_PRIVATE).getBoolean(START_AT_BOOT, false)
    }
}
