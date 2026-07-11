//! Pattern matching (spec 15.10). The flavor is the regex crate's, the
//! RE2 family: linear time, no backreferences or lookaround, and the
//! compile error names the missing feature. A match is data (Match),
//! absence is an option, offsets are character indices so they compose
//! with slicing (ADR 0019).

use std::sync::Arc;

use indexmap::IndexMap;

use crate::interp::{Fault, Interp};
use crate::value::{ErrVal, Value};

const MATCH_FIELDS: &[(&str, Kind)] = &[
    ("text", Kind::Str),
    ("start", Kind::Int),
    ("end", Kind::Int),
    ("groups", Kind::ListStr),
];

enum Kind {
    Str,
    Int,
    ListStr,
}

pub(crate) fn struct_types() -> Vec<(String, Vec<(String, crate::types::Type)>)> {
    use crate::types::Type;
    vec![
        ("Re".into(), vec![]),
        (
            "Match".into(),
            MATCH_FIELDS
                .iter()
                .map(|(f, k)| {
                    let t = match k {
                        Kind::Str => Type::Str,
                        Kind::Int => Type::Int,
                        Kind::ListStr => Type::List(Box::new(Type::Str)),
                    };
                    (f.to_string(), t)
                })
                .collect(),
        ),
    ]
}

pub(crate) fn struct_exprs() -> Vec<(String, Vec<(String, crate::ast::TypeExpr)>)> {
    use crate::ast::TypeExpr;
    let named = |n: &str| TypeExpr::Named(n.into());
    vec![(
        "Match".into(),
        MATCH_FIELDS
            .iter()
            .map(|(f, k)| {
                let t = match k {
                    Kind::Str => named("str"),
                    Kind::Int => named("int"),
                    Kind::ListStr => TypeExpr::List(Box::new(named("str"))),
                };
                (f.to_string(), t)
            })
            .collect(),
    )]
}

pub fn call(interp: &mut Interp, name: &str, args: Vec<Value>) -> Result<Value, Fault> {
    match (name, args.as_slice()) {
        ("compile", [Value::Str(pattern)]) => Ok(match regex::Regex::new(pattern) {
            Ok(re) => Value::Tuple(vec![Value::Re(Arc::new(re)), Value::NoneV]),
            Err(e) => Value::Tuple(vec![
                Value::Re(Arc::new(regex::Regex::new("$^").unwrap())),
                Value::Err(ErrVal {
                    msg: format!("regex.compile: {e}"),
                    ..Default::default()
                }),
            ]),
        }),
        _ => Err(interp.fault(format!("regex.{name}: bad arguments"))),
    }
}

/// Byte offset -> character index, so Match composes with slicing.
fn char_index(s: &str, byte: usize) -> i64 {
    s[..byte].chars().count() as i64
}

fn match_value(s: &str, c: &regex::Captures) -> Value {
    let whole = c.get(0).expect("group 0 always participates");
    let groups: Vec<Value> = (1..c.len())
        .map(|i| Value::Str(c.get(i).map_or(String::new(), |m| m.as_str().to_string())))
        .collect();
    let mut fields = IndexMap::new();
    fields.insert("text".into(), Value::Str(whole.as_str().to_string()));
    fields.insert("start".into(), Value::Int(char_index(s, whole.start())));
    fields.insert("end".into(), Value::Int(char_index(s, whole.end())));
    fields.insert("groups".into(), Value::list(groups));
    Value::Struct {
        name: "Match".into(),
        fields,
    }
}

pub fn method(
    interp: &mut Interp,
    re: &regex::Regex,
    name: &str,
    args: Vec<Value>,
) -> Result<Value, Fault> {
    let v = match (name, args.as_slice()) {
        ("matches", [Value::Str(s)]) => Value::Bool(re.is_match(s)),
        ("find", [Value::Str(s)]) => match re.captures(s) {
            Some(c) => match_value(s, &c),
            None => Value::NoneV,
        },
        ("find_all", [Value::Str(s)]) => {
            Value::list(re.captures_iter(s).map(|c| match_value(s, &c)).collect())
        }
        ("replace", [Value::Str(s), Value::Str(repl)]) => {
            Value::Str(re.replace_all(s, repl.as_str()).to_string())
        }
        _ => return Err(interp.fault(format!("Re has no method {name} with those arguments"))),
    };
    Ok(v)
}
