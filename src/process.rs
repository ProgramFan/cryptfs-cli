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
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut messages = Vec::new();
        if !stderr.trim().is_empty() {
            messages.push(format!("stderr: {}", stderr.trim()));
        }
        if !stdout.trim().is_empty() {
            messages.push(format!("stdout: {}", stdout.trim()));
        }
        if messages.is_empty() {
            messages.push(format!("exit status: {}", output.status));
        }

        bail!("`{desc}` failed: {}", messages.join("\n"));
    }
}
