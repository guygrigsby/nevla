#[derive(Debug, Clone)]
pub struct Diag {
    pub msg: String,
    pub line: u32,
    pub col: u32,
    /// Source file name, stamped by the loader; None in the repl.
    pub file: Option<String>,
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{file}:")?;
        }
        write!(f, "{}:{}: {}", self.line, self.col, self.msg)
    }
}
