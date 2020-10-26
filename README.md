# twitch-log-bot

This bot connects to twitch.tv irc servers and logs channel messages.

By default, channel messages are logged to text files. However, `postgres` can be used as well.

## Installation

Rename `config-example.json` to `config.json` and edit fields.

Note: Using `postgres` is optional; therefore, leaving this field blank will skip connection attempts.

    $ sudo apt update -y
    $ sudo apt install build-essential libssl-dev pkg-config
    $ curl https://sh.rustup.rs -sSf | sh
    $ source $HOME/.cargo/env
    $ git clone https://github.com/smehlhoff/twitch-log-bot.git
    $ cd twitch-log-bot
    $ cargo build --release
    $ nohup ./target/release/twitch-log-bot &

If you want to browse log files online, check out [AWS JavaScript S3 Explorer](https://github.com/awslabs/aws-js-s3-explorer) and follow the instructions below:

    $ sudo apt install awscli
    $ aws configure
    $ aws s3 mb s3://<bucket name>
    $ crontab -e

An example crontab to sync logs every five minutes:

    */5 * * * * aws s3 sync /home/ubuntu/twitch-log-bot/logs s3://<bucket name> --content-type "text/plain; charset=utf-8" >/dev/null 2>&1

## Usage

Use whisper commands to interact with the bot:

    /w <nickname> join #channel
    /w <nickname> part #channel
    /w <nickname> list
    /w <nickname> uptime
    /w <nickname> buffer <int>
    /w <nickname> pause
    /w <nickname> unpause
    /w <nickname> shutdown

Note: The `buffer` command sets the buffer capacity for `BufWriter<W>`, while also declaring how many messages to send for each `postgres` transaction. The bot handles the buffer value dynamically and will also set a default value based on how many channels are listed in the `config.json` file. However, it may be necessary to increase this value for higher throughput channels.

## Limitations

In theory, the bot can join an infinite number of channels. However, twitch.tv will disconnect the bot when the number of messages in queue to be sent is too large.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](https://github.com/smehlhoff/twitch-log-bot/blob/master/LICENSE)