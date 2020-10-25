use chrono::prelude::*;
use regex::Regex;
use std::fmt;

use crate::lib::error;

#[derive(Clone, Debug)]
pub enum UserType {
    User,
    Moderator,
    NotSet,
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            Self::User => write!(f, "User"),
            Self::Moderator => write!(f, "Moderator"),
            Self::NotSet => write!(f, "NotSet"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Message {
    pub command: String,
    pub target: String,
    pub user_id: i32,
    pub user_type: UserType,
    pub username: String,
    pub sub_count: i32,
    pub system_msg: String,
    pub user_msg: String,
    pub timestamp: chrono::DateTime<Utc>,
}

const fn check_user_type(user_type: i32) -> Result<UserType, error::Error> {
    Ok(match user_type {
        1 => UserType::Moderator,
        _ => UserType::User,
    })
}

impl Message {
    fn new() -> Self {
        Self {
            command: String::from(""),
            target: String::from(""),
            user_id: 0,
            user_type: UserType::NotSet,
            username: String::from(""),
            sub_count: 0,
            system_msg: String::from(""),
            user_msg: String::from(""),
            timestamp: Utc::now(),
        }
    }

    fn parse_whisper(raw_msg: &irc::proto::Message) -> Result<Self, error::Error> {
        lazy_static! {
            static ref RE: Regex = {
                let pattern = [
                    r"user-id=(?P<user_id>\d*).+",
                    r":(?P<username>\w*)!\w*@\w*.tmi.twitch.tv\s",
                    r"(?P<command>WHISPER)\s",
                    r"(?P<target>\w*)\s",
                    r":(?P<user_msg>.+)",
                ]
                .join("");

                Regex::new(&pattern).unwrap()
            };
        }

        RE.captures(&raw_msg.to_string()).map_or(Ok(Self::new()), |data| {
            Ok(Self {
                command: data["command"].to_string(),
                target: data["target"].to_string(),
                user_id: data["user_id"].to_string().parse::<i32>().unwrap_or(0),
                user_type: UserType::NotSet,
                username: data["username"].to_string(),
                sub_count: 0,
                system_msg: String::from(""),
                user_msg: data["user_msg"].replace("\r", ""),
                timestamp: Utc::now(),
            })
        })
    }

    fn parse_privmsg(raw_msg: &irc::proto::Message) -> Result<Self, error::Error> {
        lazy_static! {
            static ref RE: Regex = {
                let pattern = [
                    r"(?:@badge-info=subscriber/(?P<sub_count>\d*))?.+",
                    r"mod=(?P<user_type>\d*).+",
                    r"user-id=(?P<user_id>\d*).+",
                    r":(?P<username>\w*)!\w*@\w*.tmi.twitch.tv\s",
                    r"(?P<command>PRIVMSG)\s",
                    r"(?P<target>#\w*)\s",
                    r":(?P<user_msg>.+)",
                ]
                .join("");

                Regex::new(&pattern).unwrap()
            };
        }

        if let Some(data) = RE.captures(&raw_msg.to_string()) {
            let sub_count = data.get(1).map_or("0", |x| x.as_str());
            let user_type = check_user_type(data["user_type"].parse::<i32>().unwrap_or(0))?;

            Ok(Self {
                command: data["command"].to_string(),
                target: data["target"].to_string(),
                user_id: data["user_id"].to_string().parse::<i32>().unwrap_or(0),
                user_type,
                username: data["username"].to_string(),
                sub_count: sub_count.to_string().parse::<i32>().unwrap_or(0),
                system_msg: String::from(""),
                user_msg: data["user_msg"].replace("\r", ""),
                timestamp: Utc::now(),
            })
        } else {
            Ok(Self::new())
        }
    }

    fn parse_notice(raw_msg: &irc::proto::Message) -> Result<Self, error::Error> {
        lazy_static! {
            static ref RE: Regex = {
                let pattern = [
                    r"(?:@badge-info=subscriber/(?P<sub_count>\d*))?.+",
                    r"login=(?P<username>\w*).+",
                    r"mod=(?P<user_type>\d*).+",
                    r"room-id=(?P<room_id>\d*).+",
                    r"system-msg=(?P<system_msg>(.*?));.+",
                    r"user-id=(?P<user_id>\d*).+",
                    r"(?P<command>USERNOTICE)\s",
                    r"(?P<target>#\w*)",
                    r"(?:\s:(?P<user_msg>.+))?",
                ]
                .join("");

                Regex::new(&pattern).unwrap()
            };
        }

        if let Some(data) = RE.captures(&raw_msg.to_string()) {
            println!("{}", raw_msg);

            let user_msg = data.get(10).map_or("", |x| x.as_str());

            #[allow(clippy::trivial_regex)]
            let re = Regex::new(r"\\s")?;
            let system_msg = re.replace_all(&data["system_msg"], " ").to_string();

            // If anonymous user gifts a sub
            let (user_id, user_type, username, sub_count) = if &data["username"]
                == "ananonymousgifter"
                || &data["username"] == "ananonymouscheerer"
            {
                (0, UserType::NotSet, "anonymous".to_string(), 0)
            } else {
                let sub_count = data.get(1).map_or("0", |x| x.as_str());

                (
                    data["user_id"].to_string().parse::<i32>().unwrap_or(0),
                    check_user_type(data["user_type"].parse::<i32>().unwrap_or(0))?,
                    data["username"].to_string(),
                    sub_count.to_string().parse::<i32>().unwrap_or(0),
                )
            };

            Ok(Self {
                command: data["command"].to_string(),
                target: data["target"].to_string(),
                user_id,
                user_type,
                username,
                sub_count,
                system_msg,
                user_msg: user_msg.replace("\r", ""),
                timestamp: Utc::now(),
            })
        } else {
            Ok(Self::new())
        }
    }

    pub fn parse_msg(raw_msg: &irc::proto::Message) -> Result<Self, error::Error> {
        lazy_static! {
            static ref RE: Regex = {
                let pattern = [r"(?P<command>WHISPER|PRIVMSG|USERNOTICE)"].join("");

                Regex::new(&pattern).unwrap()
            };
        }

        if let Some(data) = RE.captures(&raw_msg.to_string()) {
            match data["command"].as_ref() {
                "WHISPER" => return Self::parse_whisper(raw_msg),
                "PRIVMSG" => return Self::parse_privmsg(raw_msg),
                "USERNOTICE" => return Self::parse_notice(raw_msg),
                _ => return Ok(Self::new()),
            }
        }

        Ok(Self::new())
    }
}
