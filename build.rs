//! Stamp the linked CPython's version at build time. The runtime
//! alternative (Py_GetVersion before init) returned an empty string on
//! 2026-07-10 GitHub runners; the build-time answer is the semantic
//! truth anyway: the python this binary was built against.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        // no python in the playground; the bridge is stubbed there
        println!("cargo:rustc-env=NEVLA_EMBEDDED_PY=none");
        return;
    }
    let v = pyo3_build_config::get().version();
    println!("cargo:rustc-env=NEVLA_EMBEDDED_PY={}.{}", v.major, v.minor);
}
