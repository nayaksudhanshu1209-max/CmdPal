//! Safe execution: program and each argument are passed separately (no shell, no string
//! concatenation of a command line). `{input}` is substituted inside a single argument.
use crate::core::settings::UserCommand;
use std::process::Command;

pub fn build_args(cmd: &UserCommand, input: &str) -> Vec<String> {
    cmd.args.iter().map(|a| a.replace("{input}", input)).collect()
}

/// Human-readable preview shown in the confirmation dialog.
pub fn preview(cmd: &UserCommand, input: &str) -> String {
    let mut s = format!("{}", cmd.program);
    for a in build_args(cmd, input) {
        if a.contains(' ') {
            s.push_str(&format!(" \"{a}\""));
        } else {
            s.push_str(&format!(" {a}"));
        }
    }
    s
}

pub fn run(cmd: &UserCommand, input: &str) -> Result<(), String> {
    if cmd.program.trim().is_empty() {
        return Err("command has no program".into());
    }
    let mut c = Command::new(cmd.program.trim());
    c.args(build_args(cmd, input));
    if !cmd.working_dir.trim().is_empty() {
        c.current_dir(cmd.working_dir.trim());
    }
    c.spawn().map(|_| ()).map_err(|e| format!("Could not start '{}': {e}", cmd.program))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd() -> UserCommand {
        UserCommand { id: "x".into(), name: "X".into(), program: "tool.exe".into(), args: vec!["-n".into(), "{input}".into(), "--tag={input}".into()], ..Default::default() }
    }

    #[test]
    fn input_stays_in_one_argument() {
        let a = build_args(&cmd(), "a b; rm -rf \"c\"");
        assert_eq!(a.len(), 3);
        assert_eq!(a[1], "a b; rm -rf \"c\"");
        assert_eq!(a[2], "--tag=a b; rm -rf \"c\"");
    }

    #[test]
    fn preview_quotes_spaces() {
        assert!(preview(&cmd(), "a b").contains("\"a b\""));
    }

    #[test]
    fn empty_program_rejected() {
        let mut c = cmd();
        c.program = " ".into();
        assert!(run(&c, "").is_err());
    }
}
