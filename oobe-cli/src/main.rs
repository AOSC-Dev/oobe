mod i18n;

use std::error::Error;

use common::{
    Locale, OobeConfig, SwapFile, USERNAME_BLOCKLIST, apply, get_recommend_swap_size, langs,
    parser::list_zoneinfo,
};
use i18n_embed::DesktopLanguageRequester;
use inquire::{
    Confirm, CustomType, Password, PasswordDisplayMode, Select, Text, required,
    validator::Validation,
};
use owo_colors::OwoColorize;
use unic_langid::LanguageIdentifier;

use crate::i18n::LANGUAGE_LOADER;

fn welcome_banner() -> anyhow::Result<()> {
    fn cell_width(c: char) -> usize {
        return match c {
            // See Wikipedia for a listing of ranges.
            '\t' => 8,
            '\u{0000}'..'\u{02af}' => 1,
            // Treat characters in General Punctuation as half-width.
            '\u{2000}'..'\u{206f}' => 1,
            '\u{ff61}'..'\u{ffdc}' => 1,
            '\u{ffe8}'..'\u{ffef}' => 1,
            _ => 2
        }
    }
    const BOX_CHARS: &[char] = &['═', '║', '╔', '╗', '╚', '╝'];
    const WIDTH: usize = 80;
    const TEXT_WIDTH: usize = WIDTH - 4;
    // static BOX_CHARS: &[char] = &['─', '│', '┌', '┐', '└', '┘'];
    let text = fl!("welcome-banner");
    let mut rendered = format!("{}{}{}\n",
        BOX_CHARS[2],
        vec![BOX_CHARS[0]; WIDTH - 2].into_iter().collect::<String>(),
        BOX_CHARS[3]
    );
    // Make every line less than TEXT_WIDTH - 4 cells
    // Screw it, I don't care if it breaks words.
    for line in text.lines() {
        let mut wrapped = String::new();
        let mut count = 0;
        for c in line.chars() {
            let w = cell_width(c);
            if count + w >TEXT_WIDTH {
                wrapped.push('\n');
                count = w;
                wrapped.push(c);
                continue;
            }
            count += w;
            wrapped.push(c);
        }
        wrapped.push_str(&vec![' '; TEXT_WIDTH - count].into_iter().collect::<String>());
        for line in wrapped.lines() {
            rendered.push_str(&format!("{} {} {}\n", BOX_CHARS[1], line, BOX_CHARS[1]));
        }
    }
    rendered.push_str(&format!("{}{}{}",
        BOX_CHARS[4],
        vec![BOX_CHARS[0]; WIDTH - 2].into_iter().collect::<String>(),
        BOX_CHARS[5]
    ));

    println!("{}", rendered.on_blue().bright_white());

    Ok(())
}

// https://manpages.ubuntu.com/manpages/oracular/en/man5/hostname.5.html
fn validate_hostname(input: &str) -> std::result::Result<Validation, Box<dyn Error + Send + Sync>> {
    if input.len() > 64 {
        return Ok(Validation::Invalid(
            fl!("hostname-illegal-too-loong").into(),
        ));
    }

    for i in ['-', '.'] {
        if input.starts_with(i) {
            return Ok(Validation::Invalid(
                fl!("hostname-illegal-starts-with", c = i.to_string()).into(),
            ));
        }
    }

    for i in ['-', '.'] {
        if input.ends_with(i) {
            return Ok(Validation::Invalid(
                fl!("hostname-illegal-ends-with", c = i.to_string()).into(),
            ));
        }
    }

    let mut is_dot = false;
    for c in input.chars() {
        if c == '.' && is_dot {
            return Ok(Validation::Invalid(
                fl!("hostname-illegal-double-dot").into(),
            ));
        } else if is_dot {
            is_dot = false;
        }

        if c == '.' {
            is_dot = true;
        }

        if !c.is_ascii_alphanumeric() && c != '-' && c != '.' {
            return Ok(Validation::Invalid(
                fl!("hostname-illegal", c = c.to_string()).into(),
            ));
        }
    }

    Ok(Validation::Valid)
}

fn validate_username(input: &str) -> std::result::Result<Validation, Box<dyn Error + Send + Sync>> {
    if USERNAME_BLOCKLIST.contains(input) {
        return Ok(Validation::Invalid(fl!("system-username").into()));
    }

    if input.starts_with(|x: char| x.is_ascii_digit()) {
        return Ok(Validation::Invalid(
            fl!("username-illegal-starts-with-number").into(),
        ));
    }

    for i in input.chars() {
        if !i.is_ascii_lowercase() && !i.is_ascii_digit() {
            return Ok(Validation::Invalid(
                fl!("username-illegal", c = i.to_string()).into(),
            ));
        }
    }

    Ok(Validation::Valid)
}

fn vaildation_fullname(
    input: &str,
) -> std::result::Result<Validation, Box<dyn Error + Send + Sync>> {
    if input.contains(":") {
        return Ok(Validation::Invalid(fl!("fullname-illegal").into()));
    }

    Ok(Validation::Valid)
}

fn get_default_username(fullname: &str) -> String {
    let mut default_username = String::new();
    let mut not_a_number = false;

    for c in fullname.chars() {
        if c.is_ascii_digit() && !not_a_number {
            continue;
        }

        if !c.is_ascii_alphabetic() && !c.is_ascii_digit() {
            continue;
        }

        default_username.push(c.to_ascii_lowercase());
        not_a_number = true;
    }

    default_username
}

fn main() -> anyhow::Result<()> {
    let localizer = crate::i18n::localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!("Error while loading languages for library_fluent {error}");
    }
    // Welcome text
    println!("{}", "AOSC OS Setup Wizard - Welcome to AOSC OS!".bright_white());
    println!("{}", "Please select your language / 选择语言:".bright_white());

    // Windows Terminal doesn't support bidirectional (BiDi) text, and renders the isolate characters incorrectly.
    // This is a temporary workaround for https://github.com/microsoft/terminal/issues/16574
    // TODO: this might break BiDi text, though we don't support any writing system depends on that.
    LANGUAGE_LOADER.set_use_isolating(false);

    // Ask for language selection first
    let langs = langs()?;
    let locale = Select::new(
        &fl!("locale"),
        langs.iter().map(|x| x.text.clone()).collect::<Vec<_>>(),
         )
        .prompt()?;
    let locale = langs.iter().find(|x| x.text == locale).unwrap();
    let newlang = &locale.locale.split('.').next().unwrap();
    let newlang = if let Ok(l) = LanguageIdentifier::from_bytes(newlang.as_bytes()) {
        l
    } else {
        // fallback
        LanguageIdentifier::from_bytes(b"en_US").unwrap()
    };
    localizer.select(&[newlang])?;

    welcome_banner()?;

    let fullname = Text::new(&fl!("fullname"))
        .with_validator(vaildation_fullname)
        .prompt()?;

    let default_username = get_default_username(&fullname);

    let username_prompt = fl!("username");
    let mut username = Text::new(&username_prompt)
        .with_validator(required!(fl!("username-required")))
        .with_validator(validate_username);

    if !default_username.is_empty() {
        username = username.with_default(&default_username);
    }

    let username = username.prompt()?;

    let password = Password::new(&fl!("password"))
        .with_validator(required!(fl!("password-required")))
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_custom_confirmation_message(&fl!("confirm-password"))
        .with_custom_confirmation_error_message(&fl!("confirm-password-not-matching"))
        .prompt()?;

    let timezones = list_zoneinfo()?;

    let timezone = Select::new(&fl!("timezone"), timezones).prompt()?;

    let hostname = Text::new(&fl!("hostname"))
        .with_validator(required!(fl!("hostname-required")))
        .with_validator(validate_hostname)
        .prompt()?;

    let rtc_as_localtime = Confirm::new(&fl!("rtc-as-localtime"))
        .with_default(false)
        .prompt()?;

    let recommend_swap_file_size = get_recommend_swap_size()?;
    let swap_size = CustomType::<f64>::new(&fl!("swap-size"))
        .with_default(
            format!("{:.2}", recommend_swap_file_size / 1024.0 / 1024.0 / 1024.0)
                .parse::<f64>()
                .unwrap(),
        )
        .prompt()?;

    apply(OobeConfig {
        locale: Locale {
            locale: locale.locale.clone(),
        },
        user: username,
        pwd: password,
        fullname: Some(fullname),
        hostname,
        rtc_as_localtime,
        timezone: common::Timezone {
            data: timezone.to_string(),
        },
        swapfile: SwapFile { size: swap_size },
    })?;

    Ok(())
}
