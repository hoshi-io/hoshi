import Tauri
import UIKit
import WebKit

private struct CreateArgs: Decodable {
  let label: String
  let url: String
  let initScript: String?
}

private struct DestroyArgs: Decodable {
  let label: String
}

private func jsString(_ s: String) -> String {
  let data = try? JSONSerialization.data(withJSONObject: [s], options: [])
  guard let data = data, let arr = String(data: data, encoding: .utf8) else { return "\"\"" }
  return String(arr.dropFirst().dropLast())
}

/// Same contract as the Android HeadlessPlugin: hidden webviews that run the
/// extension init script and post the result back through `HeadlessBridge`.
class HeadlessPlugin: Plugin, WKScriptMessageHandler, WKNavigationDelegate {
  private var webviews: [String: WKWebView] = [:]
  private var initScripts: [String: String] = [:]
  private weak var tauriWebView: WKWebView?

  override func load(webview: WKWebView) {
    tauriWebView = webview
  }

  private func label(of webView: WKWebView) -> String? {
    webviews.first(where: { $0.value === webView })?.key
  }

  // The init script calls `window.HeadlessBridge.postMessage(json)` (Android's JS interface).
  private static let bridgeShim = """
    window.HeadlessBridge = {
      postMessage: function (p) { window.webkit.messageHandlers.HeadlessBridge.postMessage(String(p)); }
    };
    """

  func userContentController(
    _ userContentController: WKUserContentController, didReceive message: WKScriptMessage
  ) {
    guard let wv = message.webView, let label = label(of: wv),
      let payload = message.body as? String
    else { return }
    dispatchToRust(label: label, data: payload)
  }

  private func dispatchToRust(label: String, data: String) {
    guard let wv = tauriWebView else {
      Logger.error("HeadlessPlugin: tauri webview not loaded yet")
      return
    }
    let js = """
      window.__TAURI_INTERNALS__.invoke('notify_done', { label: \(jsString(label)), data: \(jsString(data)) })
        .catch(function (e) { console.log('[hoshi] notify_done ERR: ' + e); });
      """
    wv.evaluateJavaScript(js, completionHandler: nil)
  }

  @objc public func create(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(CreateArgs.self)
    guard let url = URL(string: args.url) else {
      invoke.reject("invalid url")
      return
    }

    DispatchQueue.main.async {
      if let old = self.webviews[args.label] { self.destroyWebView(args.label, old) }

      let config = WKWebViewConfiguration()
      config.websiteDataStore = .default()
      config.mediaTypesRequiringUserActionForPlayback = .all
      let content = WKUserContentController()
      content.addUserScript(
        WKUserScript(
          source: HeadlessPlugin.bridgeShim, injectionTime: .atDocumentStart, forMainFrameOnly: true))
      content.add(self, name: "HeadlessBridge")
      config.userContentController = content

      let wv = WKWebView(frame: CGRect(x: 0, y: 0, width: 1, height: 1), configuration: config)
      wv.alpha = 0
      wv.isUserInteractionEnabled = false
      wv.navigationDelegate = self
      if #available(iOS 16.4, *) { wv.isInspectable = true }

      // Off-screen webviews get their timers throttled, so keep it in the hierarchy.
      ViewControllerOverrides.rootViewController()?.view.addSubview(wv)

      self.webviews[args.label] = wv
      self.initScripts[args.label] = args.initScript ?? ""
      wv.load(URLRequest(url: url))
      invoke.resolveEmpty()
    }
  }

  @objc public func destroy(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(DestroyArgs.self)
    DispatchQueue.main.async {
      if let wv = self.webviews[args.label] { self.destroyWebView(args.label, wv) }
      invoke.resolveEmpty()
    }
  }

  func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
    guard let label = label(of: webView), let script = initScripts[label] else { return }
    let host = webView.url?.host ?? ""

    webView.configuration.websiteDataStore.httpCookieStore.getAllCookies { cookies in
      let header =
        cookies
        .filter { host.hasSuffix($0.domain.trimmingCharacters(in: CharacterSet(charactersIn: "."))) }
        .map { "\($0.name)=\($0.value)" }
        .joined(separator: "; ")

      DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
        guard self.webviews[label] === webView else { return }
        webView.evaluateJavaScript("window.__nativeCookies = \(jsString(header));") { _, _ in
          webView.evaluateJavaScript(script, completionHandler: nil)
        }
      }
    }
  }

  func webView(
    _ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!,
    withError error: Error
  ) {
    Logger.error("HeadlessPlugin: load failed for \(webView.url?.absoluteString ?? "?"): \(error)")
  }

  private func destroyWebView(_ label: String, _ wv: WKWebView) {
    wv.stopLoading()
    wv.navigationDelegate = nil
    wv.configuration.userContentController.removeScriptMessageHandler(forName: "HeadlessBridge")
    wv.removeFromSuperview()
    webviews.removeValue(forKey: label)
    initScripts.removeValue(forKey: label)
  }
}

@_cdecl("init_plugin_headless")
func initHeadlessPlugin() -> Plugin {
  return HeadlessPlugin()
}
