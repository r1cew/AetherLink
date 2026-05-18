/// Safe Mode — готовые системные команды.
///
/// Все команды выполняются через PowerShell / rundll32.
/// Никакого произвольного ввода от пользователя — только предустановленные действия.
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;

use serde_json::Value;

use crate::protocol::{SafeCommand, ServerResponse};

pub fn execute(command: SafeCommand, params: Value) -> ServerResponse {
    let result = match command {
        SafeCommand::Shutdown => shutdown(),
        SafeCommand::Sleep => sleep(),
        SafeCommand::Lock => lock(),
        SafeCommand::VolumeUp => volume_key(0xAFu8), // VK_VOLUME_UP
        SafeCommand::VolumeDown => volume_key(0xAEu8), // VK_VOLUME_DOWN
        SafeCommand::VolumeSet => {
            let level = params
                .get("level")
                .and_then(|v| v.as_u64())
                .unwrap_or(50)
                .min(100) as u8;
            volume_set(level)
        }
        SafeCommand::MediaPlay | SafeCommand::MediaPause => media_key(0xB3u8), // VK_MEDIA_PLAY_PAUSE
        SafeCommand::MediaNext => media_key(0xB0u8), // VK_MEDIA_NEXT_TRACK
        SafeCommand::MediaPrev => media_key(0xB1u8), // VK_MEDIA_PREV_TRACK
    };

    match result {
        Ok(msg) => ServerResponse::ok_output(msg),
        Err(e) => ServerResponse::err(e),
    }
}

// ─── Питание ──────────────────────────────────────────────────────────────────

fn shutdown() -> Result<String, String> {
    Command::new("shutdown")
        .args(["/s", "/t", "5"])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok("Выключение через 5 секунд".into())
}

fn sleep() -> Result<String, String> {
    // rundll32 powrprof.dll,SetSuspendState — самый надёжный способ на Windows
    Command::new("rundll32.exe")
        .args(["powrprof.dll,SetSuspendState", "0,1,0"])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok("Сон".into())
}

fn lock() -> Result<String, String> {
    Command::new("rundll32.exe")
        .args(["user32.dll,LockWorkStation"])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok("Экран заблокирован".into())
}

// ─── Звук ─────────────────────────────────────────────────────────────────────

/// Нажимает виртуальную клавишу через keybd_event (C# inline в PowerShell).
fn media_key(vk: u8) -> Result<String, String> {
    let script = format!(
        r#"
Add-Type -TypeDefinition @'
using System.Runtime.InteropServices;
public class KBD {{
    [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, int dwExtraInfo);
}}
'@
[KBD]::keybd_event({vk}, 0, 0, 0)
[KBD]::keybd_event({vk}, 0, 2, 0)
"#
    );
    run_ps(&script).map(|_| format!("VK 0x{vk:02X} sent"))
}

fn volume_key(vk: u8) -> Result<String, String> {
    media_key(vk)
}

/// Устанавливает громкость в % через Windows Audio Session API (C# inline).
fn volume_set(level: u8) -> Result<String, String> {
    let script = format!(
        r#"
Add-Type -TypeDefinition @'
using System.Runtime.InteropServices;
[Guid("5CDF2C82-841E-4546-9722-0CF74078229A"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IAudioEndpointVolume {{
    int _VT1(); int _VT2(); int _VT3(); int _VT4(); int _VT5(); int _VT6(); int _VT7();
    int SetMasterVolumeLevelScalar(float fLevel, System.Guid pguidEventContext);
}}
[Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
[ClassInterface(ClassInterfaceType.None)]
class MMDeviceEnumeratorCom {{}}
public class VolumeControl {{
    [DllImport("ole32.dll")] static extern int CoCreateInstance(ref System.Guid rclsid, System.IntPtr pUnkOuter, uint dwClsContext, ref System.Guid riid, [MarshalAs(UnmanagedType.IUnknown)] out object ppv);
    public static void SetVolume(float v) {{
        var CLSID = new System.Guid("BCDE0395-E52F-467C-8E3D-C4579291692E");
        var IID   = new System.Guid("5CDF2C82-841E-4546-9722-0CF74078229A");
        object obj; CoCreateInstance(ref CLSID, System.IntPtr.Zero, 1, ref IID, out obj);
        ((IAudioEndpointVolume)obj).SetMasterVolumeLevelScalar(v, System.Guid.Empty);
    }}
}}
'@
[VolumeControl]::SetVolume({:.2})
"#,
        level as f32 / 100.0
    );
    run_ps(&script).map(|_| format!("Громкость: {level}%"))
}

// ─── Вспомогательные ─────────────────────────────────────────────────────────

fn run_ps(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell");

    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-WindowStyle",
        "Hidden",
        "-Command",
        script,
    ]);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let out = cmd.output().map_err(|e| e.to_string())?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() {
            format!("PowerShell exit {}", out.status)
        } else {
            err
        })
    }
}
