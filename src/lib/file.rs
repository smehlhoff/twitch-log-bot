use std::fs;
use std::io::{BufWriter, Write};
use std::path;

use crate::lib::{error, message};

#[derive(Debug)]
pub struct Logger {
    pub file: std::fs::File,
}

impl Logger {
    pub fn new(path: &str) -> Result<Self, error::Error> {
        let file = fs::OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self { file })
    }

    pub fn save_admin_txt(parsed_msg: &message::Message) -> Result<(), error::Error> {
        let path = format!("logs/admin/{}.txt", parsed_msg.timestamp.format("%Y-%m-%d"));
        let logger = Self::new(&path)?;
        let mut file = BufWriter::new(logger.file);

        file.write_fmt(format_args!(
            "{} - {}: {}\n",
            parsed_msg.timestamp.format("%Y-%m-%d %H:%M:%S"),
            parsed_msg.username.to_string(),
            parsed_msg.user_msg.to_string()
        ))?;

        Ok(())
    }

    pub fn save_msg_txt(parsed_msg: &message::Message, buffer: usize) -> Result<(), error::Error> {
        let path = format!(
            "logs/{}/{}.txt",
            parsed_msg.target.replace("#", ""),
            parsed_msg.timestamp.format("%Y-%m-%d")
        );
        let logger = Self::new(&path)?;
        let mut file = BufWriter::with_capacity(buffer, logger.file);

        if parsed_msg.system_msg.is_empty() {
            let moderator = match parsed_msg.user_type {
                message::UserType::Moderator => format!("[{}]", parsed_msg.user_type),
                _ => "".to_string(),
            };

            file.write_fmt(format_args!(
                "{} - {}[{}] {}: {}\n",
                parsed_msg.timestamp.format("%Y-%m-%d %H:%M:%S"),
                moderator,
                parsed_msg.sub_count,
                parsed_msg.username,
                parsed_msg.user_msg
            ))?;
        } else if !parsed_msg.system_msg.is_empty() && !parsed_msg.user_msg.is_empty() {
            file.write_fmt(format_args!(
                "{} - [Notice] {}\n{} - [Subscription Message] {}\n",
                parsed_msg.timestamp.format("%Y-%m-%d %H:%M:%S"),
                parsed_msg.system_msg,
                parsed_msg.timestamp.format("%Y-%m-%d %H:%M:%S"),
                parsed_msg.user_msg
            ))?;
        } else {
            file.write_fmt(format_args!(
                "{} - [Notice] {}\n",
                parsed_msg.timestamp.format("%Y-%m-%d %H:%M:%S"),
                parsed_msg.system_msg,
            ))?;
        }

        Ok(())
    }
}

pub fn create_dirs(channels: &[String]) -> Result<(), error::Error> {
    fs::create_dir_all(path::Path::new("logs/admin/"))?;

    for channel in channels.iter() {
        fs::create_dir_all(path::Path::new("logs/").join(channel.replace("#", "")))?;
    }

    Ok(())
}
