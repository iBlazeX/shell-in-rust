use std::mem;

pub struct ParsedCmd {
    pub cmd: String,
    pub args: Vec<String>,
    pub stout: Option<String>,
    pub sterr: Option<String>,
    pub append: bool,
}

pub fn tokenize(line: &str) -> ParsedCmd {
    let mut token = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_db_quotes = false;
    let mut backslash = false;
    let mut expect_stdout = false;
    let mut expect_stderr = false;
    let mut stout = None;
    let mut sterr = None;
    let mut append = false;
    let mut saw_redirect = false;

    for c in line.chars() {
        if backslash {
            current.push(c);
            backslash = false;
            continue;
        }
        match c {
            '\\' => {
                if in_quotes {
                    current.push(c);
                } else {
                    backslash = true;
                }
            }
            '\'' => {
                if !in_db_quotes {
                    in_quotes = !in_quotes;
                } else {
                    current.push(c);
                }
            }
            '\"' => {
                if !in_quotes {
                    in_db_quotes = !in_db_quotes;
                } else {
                    current.push(c);
                }
            }
            ' ' => {
                if in_quotes || in_db_quotes {
                    current.push(c);
                } else if !current.is_empty() {
                    if expect_stdout {
                        stout = Some(mem::take(&mut current));
                        expect_stdout = false;
                    } else if expect_stderr {
                        sterr = Some(mem::take(&mut current));
                        expect_stderr = false;
                    } else {
                        token.push(mem::take(&mut current));
                    }
                }
            }
            '>' => {
                if !in_quotes && !in_db_quotes && !backslash {
                    if saw_redirect {
                        append = true;
                    }

                    if current == "1" {
                        current.clear();
                        expect_stdout = true;
                    } else if current == "2" {
                        current.clear();
                        expect_stderr = true;
                    } else {
                        if !current.is_empty() {
                            token.push(mem::take(&mut current));
                        } else {
                            saw_redirect = !saw_redirect;
                            current.clear();
                            expect_stdout = true;
                        }
                    }
                } else {
                    current.push(c);
                }
            }
            _ => current.push(c),
        }
    }
    if backslash {
        current.push('\\');
    }
    if expect_stdout {
        stout = Some(current);
    } else if expect_stderr {
        sterr = Some(current);
    } else if !current.is_empty() {
        token.push(current);
    }
    let cmd = token.remove(0);
    ParsedCmd {
        cmd,
        args: (token),
        stout,
        sterr,
        append,
    }
}
