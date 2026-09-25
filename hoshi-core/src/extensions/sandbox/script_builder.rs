use std::sync::Arc;

use crate::extensions::SANDBOX_BOOTSTRAP;
use crate::extensions::types::{CompatLayer, ExtensionType};

pub(super) fn build_sandbox_script(
    base_classes: &str,
    compat_layer: Option<CompatLayer>,
    extension_code: &str,
    function_name: &str,
    args_json: &str,
    settings_json: &str,
    ext_type: &ExtensionType,
) -> String {
    let ext_code_repr = serde_json::to_string(extension_code).unwrap_or_default();

    let (compat_js, runner) = match &compat_layer {
        Some(CompatLayer::Lnreader(js)) => {
            let runner = format!(
                r#"(async () => {{
                    const src = {ext_repr};
                    eval(src);
                    const ExtClass = __lnr_buildNovelClass();
                    const instance = new ExtClass();
                    const fn_name = "{fn}";
                    if (typeof instance[fn_name] !== "function")
                        throw new Error(`Method "${{fn_name}}" not found on compat class`);
                    return await instance[fn_name](...{args});
                }})()"#,
                ext_repr = ext_code_repr,
                fn       = function_name,
                args     = args_json,
            );
            (js.clone(), runner)
        }

        Some(CompatLayer::Sora(js)) => {
            let build_call = match ext_type {
                ExtensionType::Novel => "__sora_buildNovelClass()",
                _                     => "__sora_buildAnimeClass()", // default/fallback
            };

            let runner = format!(
                r#"(async () => {{
                    const src = {ext_repr};
                    (0, eval)(src);

                    if (typeof globalThis.fetchv2 !== "function") {{
                        globalThis.fetchv2 = async (url, headers = {{}}, method = "GET", body = null) => {{
                            return await fetch(url, {{ method, headers, body }});
                        }};
                    }}

                    const ExtClass = {build_call};
                    const instance = new ExtClass();

                    const fn_name = "{fn}";
                    if (typeof instance[fn_name] !== "function")
                        throw new Error(`Method "${{fn_name}}" not found on Sora compat class`);

                    return await instance[fn_name](...{args});
                }})()"#,
                ext_repr   = ext_code_repr,
                build_call = build_call,
                fn         = function_name,
                args       = args_json,
            );
            (js.clone(), runner)
        }

        None => {
            let runner = format!(
                r#"(async () => {{
                    const VALID_BASES = ["Base", "Anime", "Manga", "Novel"];

                    const src            = {ext_repr};
                    const classNameMatch = src.match(/class\s+([a-zA-Z0-9_]+)\s+extends\s+([a-zA-Z0-9_]+)/);
                    if (!classNameMatch) throw new Error("No class extending a base was found in the extension");

                    const [, className, parentName] = classNameMatch;
                    if (!VALID_BASES.includes(parentName))
                        throw new Error(`Class must extend one of: ${{VALID_BASES.join(", ")}}. Got: ${{parentName}}`);

                    const ExtClass = new Function("Base", "Anime", "Manga", "Novel", `${{src}}
return ${{className}};`)(Base, Anime, Manga, Novel);

                    if (typeof ExtClass !== "function")
                        throw new Error(`Class '${{className}}' could not be loaded`);

                    const instance = new ExtClass();
                    const callable = typeof instance[`_{fn}`] === "function" ? `_{fn}` : "{fn}";

                    if (typeof instance[callable] !== "function")
                        throw new Error(`Method "{fn}" does not exist on ${{className}}`);

                    return await instance[callable](...{args});
                }})()"#,
                ext_repr = ext_code_repr,
                fn       = function_name,
                args     = args_json,
            );
            (Arc::from(""), runner)
        }
    };

    format!(
        r#"
{bootstrap}

globalThis.__settings = Object.freeze({settings});

{base}

{compat}

{runner}
"#,
        bootstrap = SANDBOX_BOOTSTRAP,
        settings  = settings_json,
        base      = base_classes,
        compat    = compat_js,
        runner    = runner,
    )
}