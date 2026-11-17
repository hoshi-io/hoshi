# App-specific R8 rules. Tauri's own rules (plugins, @Command, @InvokeArg,
# JNI bridge) come from the tauri library and proguard-tauri.pro.

# Readable stack traces in Play Console once the mapping file is applied.
-keepattributes SourceFile,LineNumberTable
-renamesourcefileattribute SourceFile

# HeadlessPlugin exposes `HeadlessBridge.postMessage` to page JavaScript.
# R8 can't see that call, so keep every @JavascriptInterface method by name.
-keepattributes JavascriptInterface
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}

# Rust implements these as Java_com_ninelfx_hoshi_PlayerSurfaceView_*,
# so the class and method names must not change.
-keep class com.ninelfx.hoshi.PlayerSurfaceView {
    native <methods>;
}

-keep class com.ninelfx.hoshi.MainActivity {
    native <methods>;
}

# rustls-platform-verifier calls its Kotlin verifier from Rust by name.
-keep, includedescriptorclasses class org.rustls.platformverifier.** { *; }

# Move every renamed (non-kept) class into one package: smaller dex.
# Kept classes, including everything JNI and Tauri look up by name, stay put.
-repackageclasses
