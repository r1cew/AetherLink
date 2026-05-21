use std::env;
use winreg::enums::*;
use winreg::RegKey;

pub fn add_to_startup() -> Result<(), Box<dyn std::error::Error>> {
    // Получаем полный путь к текущему исполняемому файлу (.exe)
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_str().unwrap();

    // Открываем ветку реестра для автозагрузки текущего пользователя
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    let (key, _) = hkcu.create_subkey(path)?;

    // Устанавливаем значение
    key.set_value("AetherLink", &exe_path_str)?;
    println!("Added to startup: {}", exe_path_str);
    Ok(())
}

pub fn remove_from_startup() -> Result<(), Box<dyn std::error::Error>> {
    // Открываем ветку реестра автозагрузки текущего пользователя
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

    let key = hkcu.open_subkey_with_flags(path, KEY_WRITE)?;

    // Удаляем значение
    key.delete_value("AetherLink")?;

    Ok(())
}
