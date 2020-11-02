#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod lib;

use indicatif::ProgressIterator;
use irc::client::prelude::*;
use lib::{config, db, file, message};
use std::sync::{Arc, Mutex};
use std::{thread, time};

fn main() {
    let mut count = 0;

    while let Err(e) = run() {
        eprintln!("{}", e);

        count += 1;
        thread::sleep(time::Duration::from_secs(30));

        if count == 3 {
            panic!("Failed to reconnect within three attempts");
        }
    }
}

fn run() -> Result<(), lib::error::Error> {
    let config = config::Config::load().expect("Unable to load config file");

    file::create_dirs(&config.channels).expect("Unable to create log directories");

    let postgres = {
        if config.postgres.is_empty() {
            false
        } else {
            match db::create_tables() {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("Logging to database is not enabled: {}", e);
                    false
                }
            }
        }
    };
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&Config {
        nickname: Some(config.nickname.to_owned()),
        server: Some(config.server.to_owned()),
        ..Config::default()
    })?;
    let count = config.channels.iter().count();
    let bot_state = Arc::new(Mutex::new(config::State::new(count, postgres)));
    let v = Arc::new(Mutex::new(Vec::new()));

    client.send(Command::Raw("PASS".to_owned(), vec![config.oauth.to_owned()], None))?;
    client.send(Command::Raw("NICK".to_owned(), vec![config.nickname.to_owned()], None))?;
    client.send(Command::Raw("CAP REQ :twitch.tv/tags".to_owned(), vec![], None))?;
    client.send(Command::Raw("CAP REQ :twitch.tv/commands".to_owned(), vec![], None))?;

    // The rate limit to join channels is 50 every 15 seconds.
    for channel in config.channels.iter().progress() {
        client.send_join(channel)?;
        thread::sleep(time::Duration::from_millis(320));
    }

    match count {
        0 => println!("Bot is now logging 0 channels..."),
        1 => println!("Bot is now logging 1 channel..."),
        _ => println!("Bot is now logging {} channels...", count),
    };

    reactor.register_client_with_handler(client, move |client, raw_msg| {
        let parsed_msg = message::Message::parse_msg(&raw_msg).expect("Unable to parse message");
        let bot_state = bot_state.lock().expect("Unable to acquire bot state mutex");
        let mut v = v.lock().expect("Unable to acquire channel mutex");

        if !parsed_msg.command.is_empty() {
            if parsed_msg.command == "WHISPER" {
                lib::commands::parse_cmd(client, bot_state, &parsed_msg)
                    .expect("Unable to save admin message")
            } else if !bot_state.paused {
                file::Logger::save_msg_txt(&parsed_msg, bot_state.buffer)
                    .expect("Unable to save message");

                if bot_state.postgres {
                    v.push(parsed_msg);

                    if v.len() >= bot_state.buffer {
                        match db::insert_logs(v.to_owned()) {
                            Ok(_) => {}
                            Err(e) => eprintln!("{}", e),
                        }

                        v.clear()
                    };
                }
            }
        };

        Ok(())
    });

    reactor.run()?;

    Ok(())
}
