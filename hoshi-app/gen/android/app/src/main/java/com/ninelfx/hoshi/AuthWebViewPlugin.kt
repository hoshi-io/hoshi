package com.ninelfx.hoshi

import android.annotation.SuppressLint
import android.app.Activity
import android.app.Dialog
import android.content.res.ColorStateList
import android.graphics.Bitmap
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.text.TextUtils
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.view.WindowManager
import android.webkit.CookieManager
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.Space
import android.widget.TextView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class OpenAuthArgs {
  lateinit var url: String
  var redirectPrefixes: Array<String> = emptyArray()
}

@TauriPlugin
class AuthWebViewPlugin(private val activity: Activity) : Plugin(activity) {

  private val accent = Color.parseColor("#6d5dfc")
  private val colBg = Color.parseColor("#0b0b0f")
  private val colBar = Color.parseColor("#14141b")
  private val colButton = Color.parseColor("#22222c")
  private val colChip = Color.parseColor("#1b1b24")
  private val colTrack = Color.parseColor("#26262f")
  private val colMuted = Color.parseColor("#b5b5c0")
  private val colWarn = Color.parseColor("#f5a524")

  private var dialog: Dialog? = null
  private var webView: WebView? = null
  private var pending: Invoke? = null

  private fun dp(v: Int) = (v * activity.resources.displayMetrics.density).toInt()

  @Command
  fun open(invoke: Invoke) {
    val args = invoke.parseArgs(OpenAuthArgs::class.java)
    activity.runOnUiThread {
      if (pending != null) {
        invoke.reject("already_open")
        return@runOnUiThread
      }
      pending = invoke
      try {
        show(args)
      } catch (e: Exception) {
        finish(null, "Error opening login: ${e.message}")
      }
    }
  }

  @Command
  fun close(invoke: Invoke) {
    activity.runOnUiThread {
      finish(null)
      invoke.resolve(JSObject())
    }
  }

  /** Resolves the pending call once, then tears the view down on the next loop turn. */
  private fun finish(url: String?, error: String? = null) {
    val inv = pending ?: return
    pending = null
    if (error != null) {
      inv.reject(error)
    } else {
      val ret = JSObject()
      if (url != null) ret.put("url", url)
      inv.resolve(ret)
    }
    // Never destroy a WebView from inside its own callback.
    Handler(Looper.getMainLooper()).post { teardown() }
  }

  private fun teardown() {
    webView?.let { wv ->
      wv.stopLoading()
      wv.webViewClient = WebViewClient()
      (wv.parent as? ViewGroup)?.removeView(wv)
      wv.destroy()
    }
    webView = null
    dialog?.dismiss()
    dialog = null
    // Don't leave the provider session behind on a shared TV.
    // NOTE: the cookie store is shared with every WebView in the app.
    CookieManager.getInstance().removeAllCookies(null)
    CookieManager.getInstance().flush()
  }


  private fun rounded(fill: Int, strokeColor: Int = Color.TRANSPARENT, strokePx: Int = 0) =
    GradientDrawable().apply {
      shape = GradientDrawable.RECTANGLE
      cornerRadius = dp(10).toFloat()
      setColor(fill)
      if (strokePx > 0) setStroke(strokePx, strokeColor)
    }

  private fun button(label: String, onClick: () -> Unit) = TextView(activity).apply {
    text = label
    textSize = 16f
    typeface = Typeface.DEFAULT_BOLD
    setTextColor(Color.WHITE)
    gravity = Gravity.CENTER
    minHeight = dp(44)
    minimumWidth = dp(44)
    setPadding(dp(18), 0, dp(18), 0)
    isFocusable = true
    isClickable = true
    background = rounded(colButton)
    layoutParams = LinearLayout.LayoutParams(
      ViewGroup.LayoutParams.WRAP_CONTENT,
      ViewGroup.LayoutParams.WRAP_CONTENT
    ).apply { marginEnd = dp(10) }
    setOnClickListener { onClick() }
    setOnFocusChangeListener { v, hasFocus ->
      v.background = if (hasFocus) rounded(accent, Color.WHITE, dp(2)) else rounded(colButton)
      val s = if (hasFocus) 1.06f else 1f
      v.animate().scaleX(s).scaleY(s).setDuration(120).start()
    }
  }

  /** Shows the real host (so people can see where they type a password); warns when not https. */
  private fun applyHost(chip: TextView, url: String?) {
    val u = Uri.parse(url ?: return)
    val host = u.host ?: return
    val secure = u.scheme == "https"
    chip.text = (if (secure) "🔒  " else "⚠  ") + host
    chip.setTextColor(if (secure) colMuted else colWarn)
    chip.visibility = View.VISIBLE
  }

  @SuppressLint("SetJavaScriptEnabled")
  private fun show(args: OpenAuthArgs) {
    val prefixes = args.redirectPrefixes.toList()
    fun isRedirect(url: String?) = url != null && prefixes.any { url.startsWith(it, ignoreCase = true) }

    // Start every login from a clean session.
    CookieManager.getInstance().removeAllCookies(null)

    val dlg = object : Dialog(activity, android.R.style.Theme_Black_NoTitleBar_Fullscreen) {
      @Suppress("OVERRIDE_DEPRECATION", "DEPRECATION")
      override fun onBackPressed() {
        val wv = webView
        if (wv != null && wv.canGoBack()) wv.goBack() else finish(null)
      }
    }

    val root = LinearLayout(activity).apply {
      orientation = LinearLayout.VERTICAL
      setBackgroundColor(colBg)
    }

    // Bar: [Close] [Back, only when there is history] [Reload] ........ [host]
    val closeBtn = button("✕  Close") { finish(null) }
    val backBtn = button("‹  Back") { webView?.goBack() }.apply { visibility = View.GONE }
    val reloadBtn = button("Reload") { webView?.reload() }

    val hostChip = TextView(activity).apply {
      textSize = 15f
      maxLines = 1
      ellipsize = TextUtils.TruncateAt.END
      gravity = Gravity.CENTER_VERTICAL
      setTextColor(colMuted)
      setPadding(dp(14), dp(8), dp(14), dp(8))
      background = rounded(colChip)
      visibility = View.INVISIBLE // until the first page starts
    }

    val bar = LinearLayout(activity).apply {
      orientation = LinearLayout.HORIZONTAL
      gravity = Gravity.CENTER_VERTICAL
      // Generous side padding: TVs often overscan the edges.
      setPadding(dp(32), dp(12), dp(32), dp(12))
      setBackgroundColor(colBar)
      addView(closeBtn)
      addView(backBtn)
      addView(reloadBtn)
      addView(Space(activity), LinearLayout.LayoutParams(0, 0, 1f))
      addView(hostChip)
    }

    // Doubles as the divider under the bar: only the dark track shows once loading is done.
    val progress = ProgressBar(activity, null, android.R.attr.progressBarStyleHorizontal).apply {
      max = 100
      progressTintList = ColorStateList.valueOf(accent)
      progressBackgroundTintList = ColorStateList.valueOf(colTrack)
      layoutParams = LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, dp(3))
    }

    val wv = WebView(activity).apply {
      settings.javaScriptEnabled = true
      settings.domStorageEnabled = true
      settings.setSupportMultipleWindows(false)
      // Some providers reject the "; wv" embedded-webview marker.
      settings.userAgentString = settings.userAgentString.replace("; wv", "")
      // Dark instead of a white flash before the first paint.
      setBackgroundColor(colBg)
      isFocusable = true
      isFocusableInTouchMode = true
      layoutParams = LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f)

      webViewClient = object : WebViewClient() {
        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
          val u = request.url.toString()
          if (isRedirect(u)) {
            finish(u)
            return true
          }
          // Keep web pages in here; refuse intent:, market:, etc.
          val scheme = request.url.scheme?.lowercase()
          return scheme != "http" && scheme != "https"
        }

        override fun onPageStarted(view: WebView, url: String?, favicon: Bitmap?) {
          if (isRedirect(url)) {
            finish(url)
            return
          }
          applyHost(hostChip, url)
        }

        override fun doUpdateVisitedHistory(view: WebView, url: String?, isReload: Boolean) {
          backBtn.visibility = if (view.canGoBack()) View.VISIBLE else View.GONE
        }

        // Safety net: an unknown-scheme redirect that slipped past shouldOverrideUrlLoading.
        override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) {
          if (request.isForMainFrame && isRedirect(request.url.toString())) finish(request.url.toString())
        }
      }

      webChromeClient = object : WebChromeClient() {
        override fun onProgressChanged(view: WebView, newProgress: Int) {
          progress.progress = if (newProgress >= 100) 0 else newProgress
        }
      }
    }

    root.addView(bar)
    root.addView(progress)
    root.addView(wv)

    dlg.setContentView(root)
    dlg.window?.apply {
      setLayout(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
      setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
    }
    // Dismissed some other way (e.g. the system): count it as cancelled.
    dlg.setOnDismissListener { finish(null) }

    dialog = dlg
    webView = wv
    dlg.show()
    wv.requestFocus()
    wv.loadUrl(args.url)
  }
}