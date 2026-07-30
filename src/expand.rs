use crate::{Shell, tokenizer::ParsedCmd};

pub fn expand_command(parsed: &mut ParsedCmd, shell: &Shell) {
    parsed.cmd = expand_vars(&parsed.cmd, shell);

    for arg in &mut parsed.args {
        *arg = expand_vars(arg, shell);
    }

    if let Some(path) = &mut parsed.stout {
        *path = expand_vars(path, shell);
    }

    if let Some(path) = &mut parsed.sterr {
        *path = expand_vars(path, shell);
    }
}

fn expand_vars(s: &str, shell: &Shell) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '$' {
            result.push(c);
            continue;
        }
        let mut name = String::new();

        if chars.peek() == Some(&'{') {
            chars.next();

            while let Some(ch) = chars.next() {
                if ch == '}' {
                    break;
                }
                name.push(ch);
            }
        } else {
            while let Some(&ch) = chars.peek() {
                if ch == '_' || ch.is_ascii_alphanumeric() {
                    name.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
        }

        if name.is_empty() {
            result.push('$');
        } else if let Some(value) = shell.vars.get(&name) {
            result.push_str(value);
        }
    }

    result
}
