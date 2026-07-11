//! The process's own surroundings (spec 15.9): working directory,
//! environment, arguments, standard input. Absence is an option or an
//! error value, never a sentinel.

use crate::interp::{Fault, Interp};
use crate::value::{ErrVal, Value};

fn err(msg: String) -> Value {
    Value::Err(ErrVal {
        msg,
        ..Default::default()
    })
}

fn fallible_str(v: Result<String, String>) -> Value {
    match v {
        Ok(s) => Value::Tuple(vec![Value::Str(s), Value::NoneV]),
        Err(m) => Value::Tuple(vec![Value::Str(String::new()), err(m)]),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn call(interp: &mut Interp, name: &str, args: Vec<Value>) -> Result<Value, Fault> {
    let v = match (name, args.as_slice()) {
        ("workdir", []) => fallible_str(
            std::env::current_dir()
                .map_err(|e| format!("workdir: {e}"))
                .and_then(|p| {
                    p.into_os_string()
                        .into_string()
                        .map_err(|_| "workdir: path is not valid unicode".to_string())
                }),
        ),
        ("env", [Value::Str(name)]) => match std::env::var(name) {
            Ok(s) => Value::Str(s),
            // unset, or set to bytes that are not unicode: both read as absent
            Err(_) => Value::NoneV,
        },
        ("args", []) => Value::list(
            interp
                .prog_args
                .iter()
                .cloned()
                .map(Value::Str)
                .collect(),
        ),
        ("readline", []) => {
            use std::io::BufRead;
            let mut line = String::new();
            match std::io::stdin().lock().read_line(&mut line) {
                Ok(0) => fallible_str(Err("eof".into())),
                Ok(_) => {
                    let line = line.trim_end_matches(['\n', '\r']).to_string();
                    fallible_str(Ok(line))
                }
                Err(e) => fallible_str(Err(format!("stdin: {e}"))),
            }
        }
        _ => return Err(interp.fault(format!("os.{name}: bad arguments"))),
    };
    Ok(v)
}

/// The browser is nobody's operating system; the module reports absence.
#[cfg(target_arch = "wasm32")]
pub fn call(interp: &mut Interp, name: &str, _args: Vec<Value>) -> Result<Value, Fault> {
    Err(interp.fault(format!("os.{name} is not available in this build")))
}
