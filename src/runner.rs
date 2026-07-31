use std::io::Write;

use crate::{builtin::run_builtin, execute::run_external, shell::Shell, tokenizer::ParsedCmd};

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
    if parsed.cmd == "exit" {
        return ShellAction::Exit;
    }

    if run_builtin(parsed, shell, out, err) {
        return ShellAction::Continue;
    }

    run_external(parsed, shell, err);

    ShellAction::Continue
}
