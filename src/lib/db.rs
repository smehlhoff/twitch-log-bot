use postgres::{Client, NoTls};
use std::thread;

use crate::lib::{config, error, message};

fn connect() -> Result<postgres::Client, error::Error> {
    let config = config::Config::load()?;
    let db = Client::connect(&config.postgres, NoTls)?;

    Ok(db)
}

pub fn create_tables() -> Result<(), error::Error> {
    let mut db = connect()?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS adminlog (
            id SERIAL PRIMARY KEY,
            user_id INT,
            username VARCHAR,
            user_msg VARCHAR,
            timestamp TIMESTAMP WITH TIME ZONE
        );",
        &[],
    )?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS chanlog (
            id SERIAL PRIMARY KEY,
            command VARCHAR,
            target VARCHAR,
            user_id INT,
            user_type VARCHAR,
            username VARCHAR,
            sub_count INT,
            system_msg VARCHAR,
            user_msg VARCHAR,
            timestamp TIMESTAMP WITH TIME ZONE
        );",
        &[],
    )?;

    Ok(())
}

pub fn insert_admin_log(log: message::Message) -> Result<(), error::Error> {
    thread::spawn(move || -> Result<(), error::Error> {
        let mut db = connect()?;

        db.execute(
            "INSERT INTO adminlog (user_id, username, user_msg, timestamp) VALUES ($1, $2, $3, $4)",
            &[&log.user_id, &log.username, &log.user_msg, &log.timestamp],
        )?;

        Ok(())
    });

    Ok(())
}

pub fn insert_logs(logs: std::vec::Vec<message::Message>) -> Result<(), error::Error> {
    thread::spawn(move || -> Result<(), error::Error> {
        let mut db = connect()?;
        let mut transaction = db.transaction()?;

        for log in &logs {
            let user_type = log.user_type.to_string();

            transaction.execute(
                "INSERT INTO chanlog (command, target, user_id, user_type, username, sub_count, system_msg, user_msg, timestamp) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[&log.command, &log.target, &log.user_id, &user_type, &log.username, &log.sub_count, &log.system_msg, &log.user_msg, &log.timestamp],
            )?;
        }

        transaction.commit()?;

        db.close()?;

        Ok(())
    });

    Ok(())
}
