import ObjectiveC
import Tauri
import UIKit

/// iOS only lets a view controller decide orientation, status bar and home indicator,
/// and the root one belongs to tao. We swap those getters on its class at runtime so the
/// orientation/immersive plugins can drive them.
enum ViewControllerOverrides {
  static var orientations: UIInterfaceOrientationMask = .allButUpsideDown
  static var statusBarHidden = false
  static var homeIndicatorHidden = false

  private static var installedOn: AnyClass?

  static func install(on vc: UIViewController) {
    let cls: AnyClass = type(of: vc)
    if installedOn === cls { return }
    installedOn = cls

    let orientationsGetter: @convention(block) (AnyObject) -> UInt = { _ in
      ViewControllerOverrides.orientations.rawValue
    }
    let statusBarGetter: @convention(block) (AnyObject) -> Bool = { _ in
      ViewControllerOverrides.statusBarHidden
    }
    let homeIndicatorGetter: @convention(block) (AnyObject) -> Bool = { _ in
      ViewControllerOverrides.homeIndicatorHidden
    }
    // Keeps the first swipe from the bottom edge from leaving the player while immersive.
    let edgesGetter: @convention(block) (AnyObject) -> UInt = { _ in
      ViewControllerOverrides.homeIndicatorHidden ? UIRectEdge.all.rawValue : 0
    }

    replace(cls, #selector(getter: UIViewController.supportedInterfaceOrientations), "Q@:", orientationsGetter)
    replace(cls, #selector(getter: UIViewController.prefersStatusBarHidden), "B@:", statusBarGetter)
    replace(cls, #selector(getter: UIViewController.prefersHomeIndicatorAutoHidden), "B@:", homeIndicatorGetter)
    replace(
      cls, #selector(getter: UIViewController.preferredScreenEdgesDeferringSystemGestures), "Q@:",
      edgesGetter)
  }

  private static func replace(_ cls: AnyClass, _ sel: Selector, _ types: String, _ block: Any) {
    let imp = imp_implementationWithBlock(block)
    class_replaceMethod(cls, sel, imp, types)
  }

  static func rootViewController() -> UIViewController? {
    let windows = UIApplication.shared.connectedScenes.flatMap { ($0 as? UIWindowScene)?.windows ?? [] }
    return (windows.first(where: { $0.isKeyWindow }) ?? windows.first)?.rootViewController
  }

  /// Applies the current values. Must run on the main thread.
  static func refresh() {
    guard let vc = rootViewController() else { return }
    install(on: vc)
    vc.setNeedsStatusBarAppearanceUpdate()
    vc.setNeedsUpdateOfHomeIndicatorAutoHidden()
    vc.setNeedsUpdateOfScreenEdgesDeferringSystemGestures()

    if #available(iOS 16.0, *) {
      vc.setNeedsUpdateOfSupportedInterfaceOrientations()
      vc.view.window?.windowScene?.requestGeometryUpdate(.iOS(interfaceOrientations: orientations)) {
        error in
        Logger.error("requestGeometryUpdate failed: \(error.localizedDescription)")
      }
    } else {
      // Pre-16 has no public API; nudging the device orientation is the accepted workaround.
      let target: UIInterfaceOrientation
      if orientations.contains(.landscapeRight) {
        target = .landscapeRight
      } else if orientations.contains(.landscapeLeft) {
        target = .landscapeLeft
      } else {
        target = .portrait
      }
      UIDevice.current.setValue(target.rawValue, forKey: "orientation")
      UIViewController.attemptRotationToDeviceOrientation()
    }
  }
}

