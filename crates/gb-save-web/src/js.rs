use gb_save_core::{PatchLogEntry, PatchLogLevel};
use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

/// Converts structured patch logs to a JS-friendly array.
#[must_use]
pub fn logs_to_js(logs: &[PatchLogEntry]) -> Array {
    let js_logs = Array::new();

    for entry in logs {
        let e = Object::new();

        let level = match entry.level {
            PatchLogLevel::Info => "info",
            PatchLogLevel::Warning => "warn",
            PatchLogLevel::Error => "error",
        };

        let class_name = match entry.level {
            PatchLogLevel::Info => "gb-save-log gb-save-log--info",
            PatchLogLevel::Warning => "gb-save-log gb-save-log--warn",
            PatchLogLevel::Error => "gb-save-log gb-save-log--error",
        };

        let _ = Reflect::set(&e, &JsValue::from_str("level"), &JsValue::from_str(level));
        let _ = Reflect::set(
            &e,
            &JsValue::from_str("className"),
            &JsValue::from_str(class_name),
        );
        let _ = Reflect::set(
            &e,
            &JsValue::from_str("source"),
            &JsValue::from_str(entry.source),
        );
        let _ = Reflect::set(
            &e,
            &JsValue::from_str("message"),
            &JsValue::from_str(&entry.message),
        );

        js_logs.push(&e);
    }

    js_logs
}

/// Builds a JavaScript object representing a patch outcome.
///
/// The returned object has the shape:
/// - `ok: boolean`
/// - `error?: string`
/// - `bytes?: Uint8Array`
/// - `logs: Array<{ level: "info" | "warn" | "error", className: string, source: string, message: string }>`
#[must_use]
pub fn patch_outcome_to_js(
    bytes: Option<&[u8]>,
    logs: &[PatchLogEntry],
    error: Option<&str>,
) -> JsValue {
    let obj = Object::new();

    let ok = error.is_none();
    let _ = Reflect::set(&obj, &JsValue::from_str("ok"), &JsValue::from_bool(ok));

    if let Some(error) = error {
        let _ = Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::from_str(error));
    }

    if let Some(out_bytes) = bytes {
        let bytes_value = Uint8Array::from(out_bytes);
        let _ = Reflect::set(&obj, &JsValue::from_str("bytes"), &bytes_value);
    }

    let _ = Reflect::set(&obj, &JsValue::from_str("logs"), &logs_to_js(logs));

    obj.into()
}
