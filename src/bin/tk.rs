//! `tk` is the runner (python parity): `tk file.mg` runs a program,
//! bare `tk` starts the REPL. Toolchain work lives in `mongoose`.
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut argv = std::env::args_os().skip(1);
    match argv.next() {
        Some(file) => {
            let args: Vec<String> = argv.map(|a| a.to_string_lossy().to_string()).collect();
            mongoose::report(mongoose::run_with(std::path::Path::new(&file), args, true))
        }
        None => {
            mongoose::repl::run();
            ExitCode::SUCCESS
        }
    }
}
