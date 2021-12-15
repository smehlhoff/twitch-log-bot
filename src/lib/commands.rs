use chrono::prelude::*;
use irc::client::prelude::*;
use std::{thread, time};

use crate::lib::{config, db, error, file, message};

fn check_prefix(mut channels: Vec<String>) -> Vec<String> {
    for channel in &mut channels {
        if !channel.contains('#') {
            channel.insert(0, '#');
        }
    }

    channels
}

fn format_channels(channels: Vec<String>) -> String {
    let channels: Vec<String> = channels.into_iter().collect();

    channels.join(" ")
}

pub fn parse_cmd(
    client: &irc::client::IrcClient,
    bot_state: std::sync::MutexGuard<config::State>,
    parsed_msg: &message::Message,
) -> Result<(), error::Error> {
    let config = config::Config::load()?;

    file::Logger::save_admin_txt(parsed_msg)?;

    if bot_state.postgres {
        match db::insert_admin_log(parsed_msg.to_owned()) {
            Ok(_) => {}
            Err(e) => eprintln!("{}", e),
        };
    };

    let mut args: Vec<String> = parsed_msg.user_msg.split(' ').map(str::to_lowercase).collect();
    let sub_cmd = &args[0];

    if config.admins.contains(&parsed_msg.username) {
        match sub_cmd.as_str() {
            "join" => {
                args.remove(0);
                join(client, bot_state, config, &parsed_msg.username, args)?;
            }
            "part" | "leave" => {
                args.remove(0);
                part(client, bot_state, config, &parsed_msg.username, args)?;
            }
            "list" | "channels" => {
                list(client, config, &parsed_msg.username)?;
            }
            "uptime" | "status" => {
                uptime(client, &bot_state, &config, &parsed_msg.username)?;
            }
            "buffer" => {
                args.remove(0);
                buffer(client, bot_state, &config, &parsed_msg.username, &args)?;
            }
            "pause" | "stop" | "unpause" | "start" => {
                pause(client, bot_state, &config, &parsed_msg.username, sub_cmd)?;
            }
            "shutdown" | "exit" | "quit" => {
                panic!("Bot shutdown by {} at {}", &parsed_msg.username, Utc::now());
            }
            _ => {}
        }
    };

    Ok(())
}

fn join(
    client: &irc::client::IrcClient,
    mut bot_state: std::sync::MutexGuard<config::State>,
    mut config: config::Config,
    admin: &str,
    channels: Vec<String>,
) -> Result<(), error::Error> {
    let channels = check_prefix(channels);
    let mut v = Vec::new();

    file::create_dirs(&channels)?;

    for channel in &channels {
        if !config.channels.contains(channel) && client.send_join(channel).is_ok() {
            v.push(channel.to_string());
            config.channels.push(channel.to_string());
            bot_state.buffer += 10;
        }
        thread::sleep(time::Duration::from_millis(320));
    }

    config.channels.sort();

    if !v.is_empty() {
        client.send(Command::Raw(
            format!("PRIVMSG {} :/w {} Joined: {}", config.nickname, admin, format_channels(v)),
            vec![],
            None,
        ))?;
    }

    config::Config::update(config)?;

    Ok(())
}

fn part(
    client: &irc::client::IrcClient,
    mut bot_state: std::sync::MutexGuard<config::State>,
    mut config: config::Config,
    admin: &str,
    channels: Vec<String>,
) -> Result<(), error::Error> {
    let channels = check_prefix(channels);
    let mut v = Vec::new();

    for channel in &channels {
        if config.channels.contains(channel) && client.send_part(channel).is_ok() {
            v.push(channel.to_string());
            config.channels.retain(|x| x != channel);
            bot_state.buffer -= 10;
        }
    }

    if !v.is_empty() {
        client.send(Command::Raw(
            format!("PRIVMSG {} :/w {} Left: {}", config.nickname, admin, format_channels(v)),
            vec![],
            None,
        ))?;
    }

    config::Config::update(config)?;

    Ok(())
}

fn list(
    client: &irc::client::IrcClient,
    config: config::Config,
    admin: &str,
) -> Result<(), error::Error> {
    let count = config.channels.len();

    match count {
        0 => {
            client.send(Command::Raw(
                format!("PRIVMSG {} :/w {} Bot is logging 0 channels", config.nickname, admin),
                vec![],
                None,
            ))?;
        }
        1 => {
            client.send(Command::Raw(
                format!(
                    "PRIVMSG {} :/w {} Bot is logging 1 channel: {}",
                    config.nickname,
                    admin,
                    format_channels(config.channels)
                ),
                vec![],
                None,
            ))?;
        }
        _ => {
            client.send(Command::Raw(
                format!(
                    "PRIVMSG {} :/w {} Bot is logging {} channels:",
                    config.nickname, admin, count
                ),
                vec![],
                None,
            ))?;

            let mut channels = config.channels.into_iter().peekable();

            while channels.peek().is_some() {
                let chunk = channels.by_ref().take(45).collect();

                client.send(Command::Raw(
                    format!("PRIVMSG {} :/w {} {}", config.nickname, admin, format_channels(chunk)),
                    vec![],
                    None,
                ))?;
            }
        }
    }

    Ok(())
}

fn uptime(
    client: &irc::client::IrcClient,
    bot_state: &config::State,
    config: &config::Config,
    admin: &str,
) -> Result<(), error::Error> {
    let current_time = Utc::now();
    let start_time = bot_state.uptime;
    let mut formatter = timeago::Formatter::new();

    formatter.num_items(3);
    formatter.ago("");

    let uptime = formatter.convert_chrono(start_time, current_time);

    client.send(Command::Raw(
        format!(
            "PRIVMSG {} :/w {} Bot uptime: {} | Bot buffer: {}",
            config.nickname, admin, uptime, bot_state.buffer
        ),
        vec![],
        None,
    ))?;

    Ok(())
}

fn buffer(
    client: &irc::client::IrcClient,
    mut bot_state: std::sync::MutexGuard<config::State>,
    config: &config::Config,
    admin: &str,
    args: &[String],
) -> Result<(), error::Error> {
    if args.is_empty() {
        client.send(Command::Raw(
            format!("PRIVMSG {} :/w {} An integer value is required", config.nickname, admin),
            vec![],
            None,
        ))?;
    } else {
        match args[0].parse::<usize>() {
            Ok(val) => {
                bot_state.buffer = val;
                client.send(Command::Raw(
                    format!(
                        "PRIVMSG {} :/w {} Bot buffer set to {}",
                        config.nickname, admin, bot_state.buffer
                    ),
                    vec![],
                    None,
                ))?;
            }
            Err(_) => {
                client.send(Command::Raw(
                    format!(
                        "PRIVMSG {} :/w {} An integer value is required",
                        config.nickname, admin
                    ),
                    vec![],
                    None,
                ))?;
            }
        }
    }

    Ok(())
}

fn pause(
    client: &irc::client::IrcClient,
    mut bot_state: std::sync::MutexGuard<config::State>,
    config: &config::Config,
    admin: &str,
    sub_cmd: &str,
) -> Result<(), error::Error> {
    if sub_cmd == "pause" || sub_cmd == "stop" {
        bot_state.paused = true;

        client.send(Command::Raw(
            format!("PRIVMSG {} :/w {} Bot logging is now paused", config.nickname, admin),
            vec![],
            None,
        ))?;
    } else {
        bot_state.paused = false;

        client.send(Command::Raw(
            format!("PRIVMSG {} :/w {} Bot logging is now unpaused", config.nickname, admin),
            vec![],
            None,
        ))?;
    }

    Ok(())
}
