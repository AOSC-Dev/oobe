// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use common::apply;
use common::parser::ZoneInfo;
use common::USERNAME_BLOCKLIST;
use serde::Serialize;
use std::env;
use std::io;
use std::process::Command;
use sysinfo::System;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

const DEFAULT_LANG: &str = "en_US.UTF-8";

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
    Ok(common::parser::list_zoneinfo()?)
}

#[tauri::command]
async fn set_config(config: &str) -> TauriResult<()> {
    Ok(apply(config)?)
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
async fn get_memory() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory = sys.total_memory();

    total_memory
}

#[tauri::command]
async fn get_recommend_swap_size() -> f64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory = sys.total_memory();

    get_recommend_swap_size_inner(total_memory)
}

pub fn get_recommend_swap_size_inner(mem: u64) -> f64 {
    const MAX_MEMORY: f64 = 32.0;

    let mem: f64 = mem as f64 / 1024.0 / 1024.0 / 1024.0;

    let res = if mem <= 1.0 {
        mem * 2.0
    } else {
        mem + mem.sqrt().round()
    };

    if res >= MAX_MEMORY {
        MAX_MEMORY * 1024.0_f32.powi(3) as f64
    } else {
        res * 1024.0_f32.powi(3) as f64
    }
}

#[tauri::command]
fn set_locale(locale: &str) {
    if let Err(e) = set_locale_inner(locale) {
        eprintln!("{e}");
    }
}

#[tauri::command]
fn exit() {
    std::process::exit(0)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        .invoke_handler(tauri::generate_handler![
            set_config,
            list_timezone,
            set_locale,
            read_locale,
            is_lang_already_set,
            is_block_username,
            get_memory,
            get_recommend_swap_size,
            exit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
