package com.ninelfx.hoshi

import android.content.pm.ActivityInfo
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

private const val TAG = "OrientationPlugin"

private fun orientationFromString(value: String): Int? = when (value.lowercase()) {
  "portrait"           -> ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
  "portrait_reverse"   -> ActivityInfo.SCREEN_ORIENTATION_REVERSE_PORTRAIT
  "landscape"          -> ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
  "landscape_reverse"  -> ActivityInfo.SCREEN_ORIENTATION_REVERSE_LANDSCAPE
  "sensor_portrait"    -> ActivityInfo.SCREEN_ORIENTATION_SENSOR_PORTRAIT
  "sensor_landscape"   -> ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
  "sensor"             -> ActivityInfo.SCREEN_ORIENTATION_SENSOR
  "unspecified"        -> ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
  else                 -> null
}

@TauriPlugin
class OrientationPlugin(private val activity: android.app.Activity) : Plugin(activity) {
  
  @Command
  fun lock(invoke: Invoke) {
    val orientationStr = invoke.getArgs().getString("orientation")

    val requestedOrientation = orientationFromString(orientationStr)
      ?: return invoke.reject("orientation desconocida: $orientationStr")

    Log.d(TAG, "lock orientation=$orientationStr ($requestedOrientation)")

    activity.runOnUiThread {
      try {
        activity.requestedOrientation = requestedOrientation
        invoke.resolve(JSObject())
      } catch (e: Exception) {
        Log.e(TAG, "Error al bloquear orientación: ${e.message}")
        invoke.reject("Error al bloquear orientación: ${e.message}")
      }
    }
  }
  
  @Command
  fun unlock(invoke: Invoke) {
    Log.d(TAG, "unlock orientation (sensor)")
    activity.runOnUiThread {
      try {
        activity.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
        invoke.resolve(JSObject())
      } catch (e: Exception) {
        Log.e(TAG, "Error al desbloquear orientación: ${e.message}")
        invoke.reject("Error al desbloquear orientación: ${e.message}")
      }
    }
  }
  
  @Command
  fun getCurrent(invoke: Invoke) {
    Log.d(TAG, "getCurrent orientation")
    activity.runOnUiThread {
      val current = activity.requestedOrientation
      val name = when (current) {
        ActivityInfo.SCREEN_ORIENTATION_PORTRAIT         -> "portrait"
        ActivityInfo.SCREEN_ORIENTATION_REVERSE_PORTRAIT -> "portrait_reverse"
        ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE        -> "landscape"
        ActivityInfo.SCREEN_ORIENTATION_REVERSE_LANDSCAPE -> "landscape_reverse"
        ActivityInfo.SCREEN_ORIENTATION_SENSOR_PORTRAIT  -> "sensor_portrait"
        ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE -> "sensor_landscape"
        ActivityInfo.SCREEN_ORIENTATION_SENSOR           -> "sensor"
        else                                             -> "unspecified"
      }
      val result = JSObject()
      result.put("orientation", name)
      invoke.resolve(result)
    }
  }
}