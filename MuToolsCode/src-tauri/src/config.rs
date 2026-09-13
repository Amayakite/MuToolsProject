use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::elevation;

pub struct AppState {
    pub log_dir: PathBuf,
    pub resource_dir: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataConfig {
    #[serde(default = "default_data_dir_mode")]
    pub data_dir_mode: String,
    #[serde(default = "default_admin_elevation")]
    pub admin_elevation: bool,
    #[serde(default = "default_aria2_max_connections")]
    pub aria2_max_connections: u32,
    #[serde(default = "default_aria2_split")]
    pub aria2_split: u32,
    #[serde(default = "default_auto_delete_installer")]
    pub auto_delete_installer: bool,
    #[serde(default = "default_update_url")]
    pub update_url: String,
    #[serde(default = "default_auto_check_update")]
    pub auto_check_update: bool,
    #[serde(default = "default_auto_refresh_mumu")]
    pub auto_refresh_mumu: bool,
    #[serde(default = "default_download_path")]
    pub download_path: String,
}

fn default_data_dir_mode() -> String { "appdata".to_string() }
fn default_admin_elevation() -> bool { true }
fn default_aria2_max_connections() -> u32 { 5 }
fn default_aria2_split() -> u32 { 5 }
fn default_auto_delete_installer() -> bool { false }
fn default_update_url() -> String { "https://mutools.netlify.app/update_info.json".to_string() }
fn default_auto_check_update() -> bool { false }
fn default_auto_refresh_mumu() -> bool { false }
fn default_download_path() -> String { default_download_dir().to_string_lossy().to_string() }

/// 获取用户的默认下载目录
fn default_download_dir() -> PathBuf {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(&profile).join("Downloads").join("MuTools")
    } else {
        PathBuf::from(".").join("MuTools")
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            data_dir_mode: "appdata".to_string(),
            admin_elevation: true,
            aria2_max_connections: 5,
            aria2_split: 5,
            auto_delete_installer: false,
            update_url: "https://mutools.netlify.app/update_info.json".to_string(),
            auto_check_update: false,
            auto_refresh_mumu: false,
            download_path: default_download_dir().to_string_lossy().to_string(),
        }
    }
}

/// 获取 exe 同目录下的配置文件路径
fn get_exe_dir_config_path() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("无法获取exe路径: {}", e))?
        .parent()
        .ok_or_else(|| "无法获取exe目录".to_string())?
        .to_path_buf();
    Ok(exe_dir.join("mutools_config.json"))
}

/// 获取 AppData 目录下的配置文件路径
fn get_appdata_config_path() -> Result<PathBuf, String> {
    Ok(resolve_appdata_dir().join("mutools_config.json"))
}

/// 解析实际使用的配置文件路径
fn get_config_path() -> Result<PathBuf, String> {
    if let Ok(exe_path) = get_exe_dir_config_path() {
        if exe_path.exists() {
            return Ok(exe_path);
        }
    }
    get_appdata_config_path()
}

pub fn read_config() -> DataConfig {
    let config_path = match get_config_path() {
        Ok(p) => p,
        Err(_) => return DataConfig::default(),
    };
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<DataConfig>(&content) {
                return config;
            }
        }
    }
    DataConfig::default()
}

/// 将配置序列化并写入实际配置文件（自动创建父目录）
fn write_config(config: &DataConfig) -> Result<(), String> {
    let config_path = get_config_path()?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

pub fn resolve_data_dir(config: &DataConfig) -> PathBuf {
    match config.data_dir_mode.as_str() {
        "exe_dir" => {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        }
        _ => {
            // "appdata" or default
            resolve_appdata_dir()
        }
    }
}

fn resolve_appdata_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("MuTools")
    } else if let Ok(home) = std::env::var("USERPROFILE") {
        PathBuf::from(home).join(".mutools")
    } else {
        PathBuf::from(".")
    }
}

#[tauri::command]
pub fn get_data_dir() -> Result<String, String> {
    let config = read_config();
    let data_dir = resolve_data_dir(&config);
    Ok(data_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_data_config(data_dir_mode: String) -> Result<(), String> {
    let existing = read_config();
    let config = DataConfig {
        data_dir_mode,
        admin_elevation: existing.admin_elevation,
        aria2_max_connections: existing.aria2_max_connections,
        aria2_split: existing.aria2_split,
        auto_delete_installer: existing.auto_delete_installer,
        update_url: existing.update_url,
        auto_check_update: existing.auto_check_update,
        auto_refresh_mumu: existing.auto_refresh_mumu,
        download_path: existing.download_path,
    };
    write_config(&config)
}

#[tauri::command]
pub fn save_admin_elevation(enabled: bool) -> Result<(), String> {
    let mut config = read_config();
    config.admin_elevation = enabled;
    write_config(&config)
}

#[tauri::command]
pub fn save_aria2_config(max_connections: u32, split: u32) -> Result<(), String> {
    let mut config = read_config();
    config.aria2_max_connections = max_connections;
    config.aria2_split = split;
    write_config(&config)
}

#[tauri::command]
pub fn get_aria2_config() -> Result<serde_json::Value, String> {
    let config = read_config();
    Ok(serde_json::json!({
        "maxConnections": config.aria2_max_connections,
        "split": config.aria2_split,
    }))
}

#[tauri::command]
pub fn check_admin_status() -> Result<bool, String> {
    Ok(elevation::current_is_admin())
}

#[tauri::command]
pub fn save_auto_delete_installer(enabled: bool) -> Result<(), String> {
    let mut config = read_config();
    config.auto_delete_installer = enabled;
    write_config(&config)
}

#[tauri::command]
pub fn get_auto_delete_installer() -> Result<bool, String> {
    let config = read_config();
    Ok(config.auto_delete_installer)
}

#[tauri::command]
pub fn save_update_config(update_url: String, auto_check_update: bool) -> Result<(), String> {
    let mut config = read_config();
    config.update_url = update_url;
    config.auto_check_update = auto_check_update;
    write_config(&config)
}

#[tauri::command]
pub fn get_update_config() -> Result<serde_json::Value, String> {
    let config = read_config();
    Ok(serde_json::json!({
        "updateUrl": config.update_url,
        "autoCheckUpdate": config.auto_check_update,
    }))
}

#[tauri::command]
pub fn save_mumu_config(auto_refresh_mumu: bool) -> Result<(), String> {
    let mut config = read_config();
    config.auto_refresh_mumu = auto_refresh_mumu;
    write_config(&config)
}

#[tauri::command]
pub fn get_mumu_config() -> Result<serde_json::Value, String> {
    let config = read_config();
    Ok(serde_json::json!({
        "autoRefreshMuMu": config.auto_refresh_mumu,
    }))
}

#[tauri::command]
pub fn save_download_config(download_path: String) -> Result<(), String> {
    let mut config = read_config();
    config.download_path = download_path;
    write_config(&config)
}

#[tauri::command]
pub fn get_download_config() -> Result<serde_json::Value, String> {
    let config = read_config();
    Ok(serde_json::json!({
        "downloadPath": config.download_path,
    }))
}

/// 检测默认安装目录
#[tauri::command]
pub fn get_default_install_dir() -> Result<String, String> {
    let d_drive = std::path::Path::new("D:\\");
    if d_drive.exists() {
        Ok("D:\\Program Files\\Netease\\MuMu".to_string())
    } else {
        Ok("C:\\Program Files\\Netease\\MuMu".to_string())
    }
}

/// 获取应用版本号
#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}

/// 通过后端请求更新信息文本 绕过前端 CORS 限制
#[tauri::command]
pub async fn fetch_update_info(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("MuTools")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status().as_u16()));
    }
    resp.text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))
}