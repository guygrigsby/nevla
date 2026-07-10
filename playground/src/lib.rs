//! wasm-bindgen surface for the playground: run a snippet, get back
//! stdout and whatever went wrong, typed by phase.

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct RunOutput {
    stdout: String,
    /// "ok" | "compile" | "runtime"
    status: &'static str,
    error: String,
}

#[wasm_bindgen]
pub fn run(source: &str) -> JsValue {
    let r = nevla::run_snippet(source);
    let (status, error) = match r.exit {
        nevla::ExitKind::Ok => ("ok", String::new()),
        nevla::ExitKind::CompileError(m) => ("compile", m),
        nevla::ExitKind::RuntimeError(m) => ("runtime", m),
    };
    let out = RunOutput {
        stdout: r.stdout,
        status,
        error,
    };
    serde_wasm_bindgen::to_value(&out).unwrap_or(JsValue::NULL)
}

#[derive(Serialize)]
struct FmtOutput {
    ok: bool,
    code: String,
    error: String,
}

#[wasm_bindgen]
pub fn fmt(source: &str) -> JsValue {
    let out = match nevla::format::fmt_source(source) {
        Ok(code) => FmtOutput {
            ok: true,
            code,
            error: String::new(),
        },
        Err(d) => FmtOutput {
            ok: false,
            code: String::new(),
            error: d.to_string(),
        },
    };
    serde_wasm_bindgen::to_value(&out).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn version() -> String {
    // the interpreter crate's version, not this shim's
    nevla::PKG_VERSION.to_string()
}
