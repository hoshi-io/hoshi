import Tauri
import UIKit

private struct LockArgs: Decodable {
  let orientation: String
}

private func mask(for value: String) -> UIInterfaceOrientationMask? {
  switch value.lowercased() {
  case "portrait": return .portrait
  case "portrait_reverse": return .portraitUpsideDown
  // Android's "landscape" is one fixed side; iOS users expect either side to work.
  case "landscape": return .landscape
  case "landscape_reverse": return .landscapeLeft
  case "sensor_portrait": return [.portrait, .portraitUpsideDown]
  case "sensor_landscape": return .landscape
  case "sensor": return .all
  case "unspecified": return .allButUpsideDown
  default: return nil
  }
}

class OrientationPlugin: Plugin {
  private var current = "unspecified"

  @objc public func lock(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(LockArgs.self)
    guard let m = mask(for: args.orientation) else {
      invoke.reject("unknown orientation: \(args.orientation)")
      return
    }
    DispatchQueue.main.async {
      self.current = args.orientation
      ViewControllerOverrides.orientations = m
      ViewControllerOverrides.refresh()
      invoke.resolve()
    }
  }

  @objc public func unlock(_ invoke: Invoke) {
    DispatchQueue.main.async {
      self.current = "unspecified"
      ViewControllerOverrides.orientations = .allButUpsideDown
      ViewControllerOverrides.refresh()
      invoke.resolve()
    }
  }

  @objc public func getCurrent(_ invoke: Invoke) {
    DispatchQueue.main.async {
      let ret: JsonObject = ["orientation": self.current]
      invoke.resolve(ret)
    }
  }
}

@_cdecl("init_plugin_orientation")
func initOrientationPlugin() -> Plugin {
  return OrientationPlugin()
}
