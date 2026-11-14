import AVFoundation
import MediaPlayer
import Tauri
import UIKit

private struct LevelArgs: Decodable {
  let level: Float
}

class ImmersivePlugin: Plugin {
  // Android's window brightness resets when the player closes; UIScreen's is global,
  // so remember the user's value and put it back on exit.
  private var originalBrightness: CGFloat?

  // iOS has no public setter for system volume. The slider inside an MPVolumeView is the
  // standard workaround, and keeping the view in the hierarchy suppresses the system HUD.
  private lazy var volumeView: MPVolumeView = {
    let v = MPVolumeView(frame: CGRect(x: -1000, y: -1000, width: 1, height: 1))
    v.alpha = 0.01
    v.isUserInteractionEnabled = false
    return v
  }()

  private var volumeSlider: UISlider? {
    if volumeView.superview == nil {
      ViewControllerOverrides.rootViewController()?.view.addSubview(volumeView)
    }
    return volumeView.subviews.compactMap { $0 as? UISlider }.first
  }

  @objc public func enter(_ invoke: Invoke) {
    DispatchQueue.main.async {
      ViewControllerOverrides.statusBarHidden = true
      ViewControllerOverrides.homeIndicatorHidden = true
      ViewControllerOverrides.refresh()
      invoke.resolve()
    }
  }

  @objc public func exit(_ invoke: Invoke) {
    DispatchQueue.main.async {
      ViewControllerOverrides.statusBarHidden = false
      ViewControllerOverrides.homeIndicatorHidden = false
      ViewControllerOverrides.refresh()
      if let b = self.originalBrightness {
        UIScreen.main.brightness = b
        self.originalBrightness = nil
      }
      invoke.resolve()
    }
  }

  @objc public func setBrightness(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(LevelArgs.self)
    DispatchQueue.main.async {
      if self.originalBrightness == nil {
        self.originalBrightness = UIScreen.main.brightness
      }
      UIScreen.main.brightness = CGFloat(min(max(args.level, 0), 1))
      invoke.resolve()
    }
  }

  @objc public func getBrightness(_ invoke: Invoke) {
    DispatchQueue.main.async {
      let ret: JsonObject = ["level": Float(UIScreen.main.brightness)]
      invoke.resolve(ret)
    }
  }

  @objc public func setVolume(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(LevelArgs.self)
    DispatchQueue.main.async {
      guard let slider = self.volumeSlider else {
        invoke.reject("volume control unavailable")
        return
      }
      // The slider only exists after the view's first layout pass.
      DispatchQueue.main.asyncAfter(deadline: .now() + 0.01) {
        slider.value = min(max(args.level, 0), 1)
        slider.sendActions(for: .valueChanged)
        invoke.resolve()
      }
    }
  }

  @objc public func getVolume(_ invoke: Invoke) {
    DispatchQueue.main.async {
      let session = AVAudioSession.sharedInstance()
      try? session.setActive(true)
      let ret: JsonObject = ["level": session.outputVolume]
      invoke.resolve(ret)
    }
  }
}

@_cdecl("init_plugin_immersive")
func initImmersivePlugin() -> Plugin {
  return ImmersivePlugin()
}
