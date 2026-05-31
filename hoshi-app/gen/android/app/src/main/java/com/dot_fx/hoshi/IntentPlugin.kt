package com.dot_fx.hoshi

import android.content.ActivityNotFoundException
import android.content.Intent
import android.content.pm.PackageManager
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import androidx.core.net.toUri

private const val TAG = "IntentPlugin"

@TauriPlugin
class IntentPlugin(private val activity: android.app.Activity) : Plugin(activity) {

  private fun findMpvPackage(pm: PackageManager): String? {
    val candidates = listOf(
      "live.mehiz.mpvkt",
      "is.xyz.mpv",
      "app.marlboroadvance.mpvex",
      "app.gyrolet.mpvrx"
    )

    for (pkg in candidates) {
      try {
        pm.getPackageInfo(pkg, 0)
        return pkg
      } catch (_: Exception) {
      }
    }

    return null
  }

  @Command
  fun launchIntent(invoke: Invoke) {
    val args = invoke.getArgs()
    val url = args.getString("url") ?: return invoke.reject("Missing url")
    val title = args.optString("title", "")
    val position = args.optInt("position", 0)
    val subsArray = args.optJSONArray("subs")
    
    val intent = Intent(Intent.ACTION_VIEW).apply {
      setDataAndType(url.toUri(), "video/any")
      
      if (title.isNotEmpty()) {
        putExtra("title", title)
      }

      if (position > 0) {
        putExtra("position", position)
      }

      if (subsArray != null && subsArray.length() > 0) {
        val subUris = Array(subsArray.length()) { i ->
          subsArray.getString(i).toUri()
        }
        putExtra("subs", subUris)
        putExtra("subs.enable", subUris)
      }
    }

    val pkg = findMpvPackage(activity.packageManager)
      ?: return invoke.reject("No supported mpv player installed")

    intent.setPackage(pkg)

    activity.runOnUiThread {
      try {
        activity.startActivity(intent)
        invoke.resolve(JSObject())
      } catch (e: ActivityNotFoundException) {
        Log.e(TAG, "mpv not installed: ${e.message}")
        invoke.reject("mpv is not installed")
      } catch (e: Exception) {
        Log.e(TAG, "Failed to launch mpv: ${e.message}")
        invoke.reject("Failed to launch mpv: ${e.message}")
      }
    }
  }
}