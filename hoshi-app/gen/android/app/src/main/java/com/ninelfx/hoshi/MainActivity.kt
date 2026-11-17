package com.ninelfx.hoshi

import android.content.Context
import android.os.Bundle
import android.graphics.Color
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    // Before any Rust code runs: HTTPS verification needs a context (see android_tls.rs).
    System.loadLibrary("hoshii_lib")
    initTls(applicationContext)
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT)
    )
    super.onCreate(savedInstanceState)
  }

  private external fun initTls(context: Context)
}