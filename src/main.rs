use std::env;
use std::fs::File;
use std::io::BufReader;

use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{self, Deserialize};

use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        id::{ChannelId, GuildId},
    },
    prelude::*,
    utils::MessageBuilder,
};

mod parse_channel_id;
mod parse_guild_id;
mod parse_regexp;

#[derive(Debug, Deserialize)]
struct Config {
    token: String,
    #[serde(with = "parse_channel_id")]
    report_channel: ChannelId,
    #[serde(with = "parse_guild_id")]
    guild: GuildId,
    filters: Vec<Filter>,
}

#[derive(Debug, Deserialize)]
struct Filter {
    #[serde(with = "parse_regexp")]
    pattern: Regex,
    note: String,
}

static CONFIG: OnceCell<Config> = OnceCell::new();

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let c = CONFIG.get().unwrap();

        if (&msg.guild_id).filter(|v| v == &c.guild).is_none() {
            return;
        }

        let notes: Vec<&str> = (&c.filters)
            .into_iter()
            .filter_map(|s| {
                if (&s).pattern.is_match(&msg.content) {
                    Some(s.note.as_str())
                } else {
                    None
                }
            })
            .collect();

        if notes.is_empty() {
            return;
        }

        // NOTE:
        // あまりに長いSPAMを送られるとそれ自身をメッセージに含むのでレポートできない可能性がある
        let msg_s = (&c.report_channel)
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("violation detected")
                        .colour(0xee0000)
                        .field(
                            "user",
                            MessageBuilder::new().mention(&msg.author.id).build(),
                            true,
                        )
                        .field(
                            "in",
                            MessageBuilder::new().mention(&msg.channel_id).build(),
                            true,
                        )
                        .field(
                            "violation(s)",
                            MessageBuilder::new().push_codeblock_safe(
                                (&notes)
                                    .into_iter()
                                    .map(|s| format!("- {}", s))
                                    .collect::<Vec<String>>()
                                    .join("\n"),
                                None,
                            ),
                            false,
                        )
                        .field(
                            "original message",
                            MessageBuilder::new().push_codeblock_safe(&msg.content, None),
                            false,
                        )
                })
            })
            .await;

        if let Err(why) = msg_s {
            println!("Error sending message: {:?}", why);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    CONFIG
        .set(
            serde_yaml::from_reader(BufReader::new(
                File::open(env::var("CONFIG").expect("Failed to lookup CONFIG environment"))
                    .expect("Failed to open CONFIG"),
            ))
            .expect("Failed to parse CONFIG"),
        )
        .unwrap();

    let mut client = Client::builder(&CONFIG.get().unwrap().token)
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
