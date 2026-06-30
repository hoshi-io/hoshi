package com.ninelfx.hoshi

import android.annotation.SuppressLint
import android.util.Log
import android.view.View
import android.view.ViewGroup
import android.webkit.ConsoleMessage
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

private const val TAG = "HeadlessPlugin"

@TauriPlugin
class HeadlessPlugin(private val activity: android.app.Activity) : Plugin(activity) {

  private val webviews = mutableMapOf<String, WebView>()

  private var tauriWebView: WebView? = null

  inner class HeadlessInterface(private val label: String) {
    @android.webkit.JavascriptInterface
    fun postMessage(payload: String) {
      Log.d(TAG, "postMessage label=$label len=${payload.length}")
      activity.runOnUiThread { dispatchToRust(label, payload) }
    }
  }

  private fun dispatchToRust(label: String, data: String) {
    val wv = tauriWebView ?: run {
      Log.e(TAG, "dispatchToRust: tauriWebView es null")
      return
    }
    val labelJson = org.json.JSONObject.quote(label)
    val dataJson  = org.json.JSONObject.quote(data)
    wv.evaluateJavascript("""
            window.__TAURI_INTERNALS__.invoke('notify_done', {
                label: $labelJson,
                data:  $dataJson
            }).catch(function(e){ console.log('[hoshi] notify_done ERR: ' + e); });
        """.trimIndent(), null)
  }

  @Command
  @SuppressLint("SetJavaScriptEnabled", "AddJavascriptInterface")
  fun create(invoke: Invoke) {
    val label      = invoke.getArgs().getString("label") ?: return invoke.reject("label requerido")
    val url        = invoke.getArgs().getString("url")   ?: return invoke.reject("url requerido")
    val initScript = invoke.getArgs().getString("initScript") ?: ""

    Log.d(TAG, "create label=$label url=$url")

    activity.runOnUiThread {
      // Capturar el WebView de Tauri una sola vez
      if (tauriWebView == null) {
        tauriWebView = findTauriWebView(activity.window.decorView as? ViewGroup)
        Log.d(TAG, "tauriWebView capturado: ${tauriWebView?.url}")
      }

      webviews[label]?.let { destroyWebView(label, it) }

      val root = activity.window.decorView as? ViewGroup ?: run {
        invoke.reject("decorView no disponible")
        return@runOnUiThread
      }

      val wv = WebView(activity).apply {
        visibility = View.VISIBLE
        alpha = 0f
        layoutParams = ViewGroup.LayoutParams(1, 1)
      }

      wv.settings.apply {
        javaScriptEnabled          = true
        domStorageEnabled          = true
        blockNetworkImage          = true
        loadsImagesAutomatically   = false
        mediaPlaybackRequiresUserGesture = true
        mixedContentMode           = android.webkit.WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
      }

      wv.webChromeClient = object : WebChromeClient() {
        override fun onConsoleMessage(message: ConsoleMessage): Boolean {
          Log.d("WV[$label]", "${message.message()} (${message.sourceId()}:${message.lineNumber()})")
          return true
        }
      }

      wv.webViewClient = object : WebViewClient() {
        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest) = false

        override fun onPageFinished(view: WebView, url: String) {
            Log.d(TAG, "onPageFinished $url")

            val cookieManager = android.webkit.CookieManager.getInstance()
            val nativeCookies = cookieManager.getCookie(url) ?: ""
            val nativeCookiesJson = org.json.JSONObject.quote(nativeCookies)

            val cookieInjection = "window.__nativeCookies = $nativeCookiesJson;"

            view.postDelayed({
                // inject native cookies first, then run the init script
                view.evaluateJavascript(cookieInjection) {
                    view.evaluateJavascript(initScript, null)
                }
            }, 500)
        }

        override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) {
          // Solo logear errores del documento principal, no de sub-recursos
          if (request.isForMainFrame) {
            Log.e(TAG, "Error en ${request.url}: ${error.description} (${error.errorCode})")
          }
        }
      }

      wv.addJavascriptInterface(HeadlessInterface(label), "HeadlessBridge")
      root.addView(wv)
      webviews[label] = wv
      wv.loadUrl(url)

      invoke.resolve(JSObject())
    }
  }

  @Command
  fun destroy(invoke: Invoke) {
    val label = invoke.getArgs().getString("label") ?: return invoke.reject("label requerido")
    Log.d(TAG, "destroy label=$label")
    activity.runOnUiThread {
      webviews[label]?.let { destroyWebView(label, it) }
      invoke.resolve(JSObject())
    }
  }

  private fun findTauriWebView(root: ViewGroup?): WebView? {
    if (root == null) return null
    for (i in 0 until root.childCount) {
      val child = root.getChildAt(i)
      if (child is WebView && !webviews.values.contains(child)) return child
      if (child is ViewGroup) findTauriWebView(child)?.let { return it }
    }
    return null
  }

  private fun destroyWebView(label: String, wv: WebView) {
    try {
      (wv.parent as? ViewGroup)?.removeView(wv)
      wv.stopLoading()
      wv.clearHistory()
      wv.destroy()
      webviews.remove(label)
      Log.d(TAG, "WebView '$label' destruido")
    } catch (e: Exception) {
      Log.e(TAG, "Error destruyendo '$label': ${e.message}")
    }
  }
}