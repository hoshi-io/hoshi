import AVFoundation
import HoshiSysLibs
import Metal
import QuartzCore
import Tauri
import UIKit
import WebKit

// Implemented in hoshi-app/src/player_surface/ios.rs
@_silgen_name("hoshi_player_surface_created")
private func hoshiPlayerSurfaceCreated(_ layer: UnsafeMutableRawPointer)

@_silgen_name("hoshi_player_surface_destroyed")
private func hoshiPlayerSurfaceDestroyed()

/// mpv draws straight into this layer (gpu-context=moltenvk, wid = the CAMetalLayer),
/// the iOS equivalent of the Android SurfaceView.
final class PlayerMetalView: UIView {
  override class var layerClass: AnyClass { CAMetalLayer.self }

  var metalLayer: CAMetalLayer { layer as! CAMetalLayer }

  override init(frame: CGRect) {
    super.init(frame: frame)
    backgroundColor = .black
    isUserInteractionEnabled = false
    autoresizingMask = [.flexibleWidth, .flexibleHeight]
    metalLayer.pixelFormat = .bgra8Unorm
    metalLayer.framebufferOnly = true
  }

  required init?(coder: NSCoder) { fatalError("init(coder:) is not supported") }

  override func layoutSubviews() {
    super.layoutSubviews()
    let scale = window?.screen.nativeScale ?? UIScreen.main.nativeScale
    metalLayer.contentsScale = scale
    metalLayer.drawableSize = CGSize(width: bounds.width * scale, height: bounds.height * scale)
  }
}

class PlayerSurfacePlugin: Plugin {
  private var surfaceView: PlayerMetalView?
  private var attached = false
  private var observers: [NSObjectProtocol] = []

  override func load(webview: WKWebView) {
    DispatchQueue.main.async { self.install(webview, attempt: 0) }
  }

  private func install(_ webview: WKWebView, attempt: Int) {
    // The webview may not be in the hierarchy yet when plugins load.
    guard let container = webview.superview else {
      if attempt < 50 {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
          self.install(webview, attempt: attempt + 1)
        }
      } else {
        Logger.error("PlayerSurfacePlugin: webview never got a superview, video will not show")
      }
      return
    }

    // Plays with the silent switch on and keeps audio going in the background.
    try? AVAudioSession.sharedInstance().setCategory(.playback, mode: .moviePlayback)

    webview.isOpaque = false
    webview.backgroundColor = .clear
    webview.scrollView.backgroundColor = .clear

    surfaceView?.removeFromSuperview()
    let view = PlayerMetalView(frame: container.bounds)
    container.insertSubview(view, belowSubview: webview)
    surfaceView = view
    view.layoutIfNeeded()

    attach()

    // GPU work from the background is refused, so drop the surface like Android does.
    let nc = NotificationCenter.default
    observers.forEach { nc.removeObserver($0) }
    observers = [
      nc.addObserver(
        forName: UIApplication.didEnterBackgroundNotification, object: nil, queue: .main
      ) { [weak self] _ in self?.detach() },
      nc.addObserver(
        forName: UIApplication.willEnterForegroundNotification, object: nil, queue: .main
      ) { [weak self] _ in self?.attach() },
    ]
  }

  private func attach() {
    guard !attached, let view = surfaceView else { return }
    attached = true
    hoshiPlayerSurfaceCreated(Unmanaged.passUnretained(view.metalLayer).toOpaque())
  }

  private func detach() {
    guard attached else { return }
    attached = false
    hoshiPlayerSurfaceDestroyed()
  }
}

@_cdecl("init_plugin_player_surface")
func initPlayerSurfacePlugin() -> Plugin {
  return PlayerSurfacePlugin()
}
