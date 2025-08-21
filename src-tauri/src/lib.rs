// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use parser::list_zoneinfo;
use parser::ZoneInfo;
use serde::Serialize;
use std::collections::HashSet;
use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::LazyLock;
use tauri::Manager;
use tauri::WindowEvent;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

use crate::utils::handle_serde_config;
use crate::utils::OobeConfig;
use crate::utils::SwapFile;
use crate::utils::Timezone;

mod parser;
mod utils;

const DEFAULT_LANG: &str = "en_US.UTF-8";
static USERNAME_BLOCKLIST: LazyLock<HashSet<&str>> =
    LazyLock::new(|| include_str!("../users").lines().collect::<HashSet<_>>());
static CAN_CLOSE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize)]
pub struct CommandError(String);

impl<E> From<E> for CommandError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into().to_string())
    }
}

type TauriResult<T> = Result<T, CommandError>;

#[tauri::command]
fn list_timezone() -> TauriResult<Vec<ZoneInfo>> {
    Ok(list_zoneinfo()?)
}

#[tauri::command]
async fn set_config(config: &str) -> TauriResult<()> {
    let OobeConfig {
        locale,
        user,
        pwd,
        fullname,
        hostname,
        rtc_as_localtime,
        timezone,
        swapfile,
    } = handle_serde_config(config)?;

    install::hostname::set_hostname(&hostname)?;
    install::locale::set_locale(&locale.locale)?;
    install::user::add_new_user(&user, &pwd)?;
    install::locale::set_hwclock_tc(!rtc_as_localtime)?;

    let SwapFile { size } = swapfile;

    if size != 0.0 {
        install::swap::create_swapfile(size, Path::new("/"))?;
    }

    if let Some(fullname) = fullname {
        install::user::passwd_set_fullname(&fullname, &user)?;
    }

    let Timezone { data: timezone } = timezone;

    install::zoneinfo::set_zoneinfo(&timezone)?;

    Ok(())
}

#[tauri::command]
async fn is_block_username(username: String) -> bool {
    USERNAME_BLOCKLIST.contains(username.as_str())
}

#[tauri::command]
async fn read_locale() -> String {
    env::var("LANG").unwrap_or_else(|_| String::from(DEFAULT_LANG))
}

#[tauri::command]
fn set_locale(locale: &str) {
    if let Err(e) = set_locale_inner(locale) {
        eprintln!("{e}");
    }
}

#[tauri::command]
/// Skip the language selection menu if LANG is set from the boot menu.
/// LANG will be C.UTF-8 if not chosen at boot.
async fn is_lang_already_set() -> bool {
    env::var("LANG").is_ok_and(|x| x != "C.UTF-8")
}

fn set_locale_inner(locale: &str) -> io::Result<()> {
    Command::new("localectl")
        .arg("set-locale")
        .arg(locale)
        .output()?;

    Ok(())
}

#[tokio::main]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // initialize tracing
    let env_log = EnvFilter::try_from_default_env();

    if let Ok(filter) = env_log {
        tracing_subscriber::registry()
            .with(fmt::layer().with_filter(filter))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(fmt::layer().with_filter(LevelFilter::INFO))
            .init();
    }

    info!("Git version: {}", env!("VERGEN_GIT_SHA"));

    // 预热用户名黑名单
    let _ = &*USERNAME_BLOCKLIST;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_cli::init())
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();

            window.on_window_event(|e| {
                if let WindowEvent::CloseRequested { api, .. } = e {
                    if !CAN_CLOSE.load(Ordering::SeqCst) {
                        api.prevent_close();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_config,
            list_timezone,
            set_locale,
            read_locale,
            is_lang_already_set,
            is_block_username,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
