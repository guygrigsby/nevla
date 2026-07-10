//! `nv` is the runner (python parity): `nv file.nv` runs a program,
//! bare `nv` starts the REPL. Toolchain work lives in `nevla`.
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut argv = std::env::args_os().skip(1);
    match argv.next() {
        Some(flag) if flag == "--version" || flag == "-V" => {
            println!(
                "nv {} (python {})",
                env!("CARGO_PKG_VERSION"),
                nevla::bridge::embedded_python()
            );
            ExitCode::SUCCESS
        }
        Some(file) => {
            let args: Vec<String> = argv.map(|a| a.to_string_lossy().to_string()).collect();
            nevla::report(nevla::run_with(std::path::Path::new(&file), args, true))
        }
        None => {
            nevla::repl::run();
            ExitCode::SUCCESS
        }
    }
}
