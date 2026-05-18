/// Developer Mode — исполнение произвольных команд в shell.
///
/// ВНИМАНИЕ: режим выключен по умолчанию.
/// Требует: device.mode == Developer.
/// Команда выполняется синхронно, stdout+stderr возвращаются в ответе.
use std::process::Command;

use crate::protocol::{ServerResponse, ShellType};

pub fn execute(cmd: String, shell: ShellType) -> ServerResponse {
    let result = match shell {
        ShellType::Powershell => run_powershell(&cmd),
        ShellType::Cmd => run_cmd(&cmd),
    };

    match result {
        Ok(output) => ServerResponse::ok_output(output),
        Err(e) => ServerResponse::err(e),
    }
}

fn run_powershell(cmd: &str) -> Result<String, String> {
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", cmd])
        .output()
        .map_err(|e| e.to_string())?;

    merge_output(out)
}

fn run_cmd(cmd: &str) -> Result<String, String> {
    let out = Command::new("cmd")
        .args(["/c", cmd])
        .output()
        .map_err(|e| e.to_string())?;

    merge_output(out)
}

fn merge_output(out: std::process::Output) -> Result<String, String> {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}").trim().to_string();

    if out.status.success() {
        Ok(combined)
    } else {
        Err(if combined.is_empty() {
            format!("Команда завершилась с кодом: {:?}", out.status.code())
        } else {
            combined
        })
    }
}
