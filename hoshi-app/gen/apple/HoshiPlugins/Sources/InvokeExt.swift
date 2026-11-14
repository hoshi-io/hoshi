import Tauri

extension Invoke {
  /// Rust deserializes "no data" replies into an empty struct, which plain `resolve()`
  /// (sent as `null`) fails. Kotlin's `resolve(JSObject())` sends `{}`; match it.
  func resolveEmpty() {
    resolve(JsonObject())
  }
}
