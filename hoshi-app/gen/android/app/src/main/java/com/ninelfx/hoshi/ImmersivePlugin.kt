package com.ninelfx.hoshi

import android.os.Build
import android.util.Log
import android.view.WindowInsets
import android.view.WindowInsetsController
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import android.view.WindowManager
import android.graphics.Color
import androidx.annotation.RequiresApi

private const val TAG = "ImmersivePlugin"

@TauriPlugin
class ImmersivePlugin(private val activity: android.app.Activity) : Plugin(activity) {

    private var originalCutoutMode: Int? = null
    private var originalSystemUiVisibility: Int? = null
    private var originalStatusBarColor: Int? = null
    private var originalNavigationBarColor: Int? = null
    private var originalWindowFlags: Int? = null

    @RequiresApi(Build.VERSION_CODES.P)
    @Command
    fun enter(invoke: Invoke) {
        activity.runOnUiThread {
            try {
                if (originalCutoutMode == null) {
                    originalCutoutMode = activity.window.attributes.layoutInDisplayCutoutMode
                }
                if (originalSystemUiVisibility == null) {
                    @Suppress("DEPRECATION")
                    originalSystemUiVisibility = activity.window.decorView.systemUiVisibility
                }
                if (originalStatusBarColor == null) {
                    originalStatusBarColor = activity.window.statusBarColor
                }
                if (originalNavigationBarColor == null) {
                    originalNavigationBarColor = activity.window.navigationBarColor
                }
                if (originalWindowFlags == null) {
                    originalWindowFlags = activity.window.attributes.flags
                }

                val attrs = activity.window.attributes
                attrs.layoutInDisplayCutoutMode =
                    WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
                activity.window.attributes = attrs

                activity.window.addFlags(WindowManager.LayoutParams.FLAG_DRAWS_SYSTEM_BAR_BACKGROUNDS)
                activity.window.clearFlags(WindowManager.LayoutParams.FLAG_TRANSLUCENT_STATUS)
                activity.window.clearFlags(WindowManager.LayoutParams.FLAG_TRANSLUCENT_NAVIGATION)

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                    activity.window.insetsController?.let {
                        it.hide(WindowInsets.Type.statusBars() or WindowInsets.Type.navigationBars())
                        it.systemBarsBehavior =
                            WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                    }
                } else {
                    @Suppress("DEPRECATION")
                    activity.window.decorView.systemUiVisibility = (
                        android.view.View.SYSTEM_UI_FLAG_FULLSCREEN
                        or android.view.View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                        or android.view.View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                        or android.view.View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                        or android.view.View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                    )
                }
                invoke.resolve(JSObject())
            } catch (e: Exception) {
                invoke.reject("Error entering immersive mode: ${e.message}")
            }
        }
    }

    @RequiresApi(Build.VERSION_CODES.P)
    @Command
    fun exit(invoke: Invoke) {
        activity.runOnUiThread {
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                    activity.window.insetsController?.show(
                        WindowInsets.Type.statusBars() or WindowInsets.Type.navigationBars()
                    )
                } else {
                    @Suppress("DEPRECATION")
                    activity.window.decorView.systemUiVisibility = originalSystemUiVisibility ?: 0
                }

                originalWindowFlags?.let { flags ->
                    val targetMask = WindowManager.LayoutParams.FLAG_DRAWS_SYSTEM_BAR_BACKGROUNDS or
                            WindowManager.LayoutParams.FLAG_TRANSLUCENT_STATUS or
                            WindowManager.LayoutParams.FLAG_TRANSLUCENT_NAVIGATION
                    activity.window.clearFlags(targetMask)
                    activity.window.addFlags(flags and targetMask)
                }

                val attrs = activity.window.attributes
                attrs.layoutInDisplayCutoutMode = originalCutoutMode ?: WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_DEFAULT
                activity.window.attributes = attrs

                originalStatusBarColor?.let { activity.window.statusBarColor = it }
                originalNavigationBarColor?.let { activity.window.navigationBarColor = it }

                invoke.resolve(JSObject())
            } catch (e: Exception) {
                invoke.reject("Error exiting immersive mode: ${e.message}")
            }
        }
    }
}