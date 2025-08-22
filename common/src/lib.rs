pub mod parser;

use std::{collections::HashSet, path::Path, sync::LazyLock};

use install::{utils::run_command, *};
use serde::Deserialize;

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

pub fn apply(config: &str) -> anyhow::Result<()> {
    let OobeConfig {
        locale,
        user,
        pwd,
        fullname,
        hostname,
        rtc_as_localtime,
        timezone,
        swapfile,
    } = serde_json::from_str(config)?;

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
