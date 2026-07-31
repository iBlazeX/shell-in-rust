use std::io::Write;

use crate::{
    builtin::{BuiltinResult, run_builtin},
    execute::run_external,
    shell::Shell,
    tokenizer::ParsedCmd,
};

pub enum ShellAction {
    Continue,
    Exit,
}

pub fn run(
    parsed: &ParsedCmd,
    shell: &mut Shell,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> ShellAction {
    match run_builtin(parsed, shell, out, err) {
        BuiltinResult::Continue => ShellAction::Continue,
        BuiltinResult::Exit => ShellAction::Exit,
        BuiltinResult::NotBuiltin => run_external(parsed, shell, err),
    }
}
