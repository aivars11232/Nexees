package dev.nexees.feasibility

import android.content.Context
import android.content.res.AssetManager
import org.json.JSONObject
import java.io.File

const val TAG = "NexeesFeasibility"

/** The shared Rust core. Every call is one JSON request and one JSON reply. */
object NativeCore {
    init {
        System.loadLibrary("nexees_feasibility")
    }

    @JvmStatic
    private external fun call(request: String): String

    fun request(context: Context, op: String, fields: JSONObject = JSONObject()): JSONObject {
        fields.put("op", op).put("files", context.filesDir.absolutePath)
        return JSONObject(call(fields.toString()))
    }
}

/**
 * Copies the bundled assets to `files/assets` once per installed APK. The LCL
 * engine verifies its Core packages as real files, and the probe reads its
 * fixtures and the LCL user manual from there.
 */
@Synchronized
fun extractAssets(context: Context) {
    val stamp = File(context.filesDir, "assets.stamp")
    val installed = context.packageManager.getPackageInfo(context.packageName, 0).lastUpdateTime.toString()
    if (stamp.isFile && stamp.readText() == installed) return
    val target = File(context.filesDir, "assets")
    target.deleteRecursively()
    copyAsset(context.assets, "nexees", target)
    stamp.writeText(installed)
}

private fun copyAsset(assets: AssetManager, path: String, target: File) {
    val children = assets.list(path).orEmpty()
    if (children.isEmpty()) {
        target.parentFile?.mkdirs()
        assets.open(path).use { input -> target.outputStream().use { input.copyTo(it) } }
    } else {
        target.mkdirs()
        children.forEach { copyAsset(assets, "$path/$it", File(target, it)) }
    }
}
