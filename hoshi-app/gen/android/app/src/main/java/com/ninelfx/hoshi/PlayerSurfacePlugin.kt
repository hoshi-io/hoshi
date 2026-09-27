package com.ninelfx.hoshi

import android.app.Activity
import android.graphics.Color
import android.graphics.PixelFormat
import android.graphics.drawable.ColorDrawable
import android.util.Log
import android.view.ViewGroup
import android.webkit.WebView
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin

private const val TAG = "PlayerSurfacePlugin"

@TauriPlugin
class PlayerSurfacePlugin(private val activity: Activity) : Plugin(activity) {

  private var surfaceView: PlayerSurfaceView? = null

  // Tauri calls this once the WebView it manages is ready. It's the
  // first point we have any view to anchor against at all — there's no
  // Android equivalent of grabbing an HWND during setup().
  override fun load(webView: WebView) {
    super.load(webView)
    webView.setBackgroundColor(Color.TRANSPARENT)

    activity.runOnUiThread {
      // A SurfaceView punches a hole through to whatever's behind the
      // *window* in the compositor. The theme's windowBackground (opaque
      // by default) and a non-translucent window pixel format both
      // block that hole from actually showing anything through it —
      // this, not something inherent to SurfaceView, was the source of
      // the earlier black-screen issue. Both must be set together.
      activity.window.setFormat(PixelFormat.TRANSLUCENT)
      activity.window.setBackgroundDrawable(ColorDrawable(Color.TRANSPARENT))

      surfaceView?.let { (it.parent as? ViewGroup)?.removeView(it) }

      val container = webView.parent as? ViewGroup
      if (container == null) {
        Log.e(TAG, "WebView has no parent ViewGroup, cannot insert PlayerSurfaceView")
        return@runOnUiThread
      }

      val view = PlayerSurfaceView(activity)
      surfaceView = view
      container.addView(view, 0, ViewGroup.LayoutParams(
        ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT
      ))
    }
  }
}