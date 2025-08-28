pub mod parser;

use std::{collections::HashSet, path::Path, sync::LazyLock};

use install::{utils::run_command, *};
use serde::{Deserialize, Serialize};
use sysinfo::System;

pub static USERNAME_BLOCKLIST: LazyLock<HashSet<&str>> =
    LazyLock::new(|| include_str!("../users").lines().collect::<HashSet<_>>());

#[derive(Deserialize)]
pub struct OobeConfig {
    pub locale: Locale,
    pub user: String,
    pub pwd: String,
    pub fullname: Option<String>,
    pub hostname: String,
    pub rtc_as_localtime: bool,
    pub timezone: Timezone,
    pub swapfile: SwapFile,
}

#[derive(Deserialize)]
pub struct SwapFile {
    pub size: f64,
}

#[derive(Deserialize)]
pub struct Locale {
    pub locale: String,
}

#[derive(Deserialize)]
pub struct Timezone {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lang {
    lang_english: String,
    pub locale: String,
    lang: String,
    pub text: String,
    data: String,
}

pub const LOCALE_LIST: &str = include_str!("../lang_select.json");

pub fn langs() -> anyhow::Result<Vec<Lang>> {
    Ok(serde_json::from_str(LOCALE_LIST)?)
}

pub fn get_recommend_swap_size() -> anyhow::Result<f64> {
    let mut res = get_recommend_swap_size_inner(get_memory());
    let available_space = fs4::available_space("/")? as f64;

    if res >= available_space {
        res = (res - available_space) / 1.25
    }

    Ok(res)
}

pub fn get_memory() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();

    sys.total_memory()
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

pub fn apply(config: OobeConfig) -> anyhow::Result<()> {
    let OobeConfig {
        locale,
        user,
        pwd,
        fullname,
        hostname,
        rtc_as_localtime,
        timezone,
        swapfile,
    } = config;

    hostname::set_hostname(&hostname)?;
    locale::set_locale(&locale.locale)?;
    user::add_new_user(&user, &pwd)?;
    locale::set_hwclock_tc(!rtc_as_localtime)?;

    let SwapFile { size } = swapfile;

    if size != 0.0 {
        swap::create_swapfile(size * 1024.0 * 1024.0 * 1024.0, Path::new("/"))?;
        genfstab::write_swap_entry_to_fstab()?;
    }

    if let Some(fullname) = fullname {
        user::passwd_set_fullname(&fullname, &user)?;
    }

    let Timezone { data: timezone } = timezone;

    zoneinfo::set_zoneinfo(&timezone)?;

    // Re-gemerate machine id
    run_command(
        "systemd-machine-id-setup",
        &[] as &[&str],
        vec![] as Vec<(String, String)>,
    )?;

    Ok(())
}
