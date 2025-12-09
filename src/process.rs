use anyhow::{bail, Context, Result};
use std::ffi::OsStr;
use std::process::{Command, Output};

pub fn format_command(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(OsStr::to_string_lossy)
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        program.to_string()
    } else {
        format!("{program} {args}")
    }
}

pub fn run_with_output(cmd: &mut Command) -> Result<Output> {
    let desc = format_command(cmd);
    let output = cmd
        .output()
        .with_context(|| format!("failed to run `{desc}`"))?;

    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`{desc}` failed: {}", stderr.trim());
    }
}
