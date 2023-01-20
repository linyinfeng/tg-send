use std::{
    io::{self, Read},
    process::exit,
};

use clap::{Parser, ValueEnum};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, Message, ParseMode},
    Bot,
};

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    pretty_env_logger::init();
    let bot = Bot::from_env();
    let options = Options::parse();

    match read_send_format(bot, &options).await {
        Ok(m) => {
            if !options.silent {
                println!("{m}");
            }
        }
        Err(e) => {
            if !options.silent {
                log::error!("{e}");
            }
            exit(1);
        }
    }
}

async fn read_send_format(bot: Bot, options: &Options) -> Result<String, Error> {
    let mut msg = String::new();
    io::stdin().read_to_string(&mut msg)?;
    send_message(bot, options, msg).await.and_then(format_msg)
}

async fn send_message(bot: Bot, options: &Options, msg: String) -> Result<Message, Error> {
    let send = bot.send_message(options.chat_id, msg);
    let send = match options.message_type {
        MessageType::PlainText => send,
        MessageType::Markdown => send.parse_mode(ParseMode::MarkdownV2),
        MessageType::Html => send.parse_mode(ParseMode::Html),
    };
    Ok(send.await?)
}

fn format_msg(msg: Message) -> Result<String, Error> {
    Ok(serde_json::to_string_pretty(&msg)?)
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Options {
    #[arg(short, long, value_parser = parse_chat_id)]
    pub chat_id: ChatId,
    #[arg(short, long, value_enum, default_value = "plain-text")]
    pub message_type: MessageType,
    #[arg(short, long)]
    pub silent: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum MessageType {
    PlainText,
    Markdown,
    Html,
}

fn parse_chat_id(s: &str) -> Result<ChatId, String> {
    let num = s.parse().map_err(|_| format!("`{s}` isn't a chat id"))?;
    Ok(ChatId(num))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request error: {0}")]
    RequestError(#[from] teloxide::RequestError),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Options::command().debug_assert()
    }
}
