package com.ninelfx.hoshi

import android.content.Context
import android.util.Log
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView

private const val TAG = "PlayerSurfaceView"

/**
 * Hands its [Surface] straight to the native side via JNI on every
 * create/destroy — see `android.rs`'s `nativeSurfaceCreated` /
 * `nativeSurfaceDestroyed` for what happens on the other end.
 *
 * Reverted from a short-lived `TextureView` experiment: `TextureView`
 * needed explicit `SurfaceTexture.setDefaultBufferSize` handling and
 * still produced visible corruption on resize/reinit, on top of its
 * inherent per-frame copy overhead. Back to `SurfaceView`'s direct
 * compositing path — the black-screen issue that originally motivated
 * moving away from it was actually the window's own opaque background
 * drawable + non-translucent pixel format, not something inherent to
 * SurfaceView; both are addressed explicitly below and in
 * `PlayerSurfacePlugin.kt`.
 *
 * No `System.loadLibrary` call here: Tauri's own bootstrap already loads
 * the app's native library before any Activity/View code runs, so these
 * `external fun`s are resolvable the moment this class exists.
 *
 * `surfaceChanged` is intentionally not forwarded — mpv reads size
 * straight off the `Surface` itself once attached, there's nothing useful
 * to tell it here (this is true for SurfaceView, unlike the TextureView
 * detour where SurfaceTexture geometry had to be pushed explicitly).
 */
class PlayerSurfaceView(context: Context) : SurfaceView(context), SurfaceHolder.Callback {

  init {
    setZOrderOnTop(false)
    holder.setFormat(android.graphics.PixelFormat.TRANSLUCENT)
    holder.addCallback(this)
  }

  override fun surfaceCreated(holder: SurfaceHolder) {
    Log.d(TAG, "surfaceCreated")
    nativeSurfaceCreated(holder.surface)
  }

  override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
    // No-op — see class doc.
  }

  override fun surfaceDestroyed(holder: SurfaceHolder) {
    Log.d(TAG, "surfaceDestroyed")
    nativeSurfaceDestroyed()
  }

  private external fun nativeSurfaceCreated(surface: Surface)
  private external fun nativeSurfaceDestroyed()
}