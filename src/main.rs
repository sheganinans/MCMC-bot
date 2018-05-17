             extern crate markov;
#[macro_use] extern crate serenity;
             extern crate quickersort;


use markov::Chain;

use serenity::builder::GetMessages;
use serenity::client::Client;
use serenity::framework::StandardFramework;
use serenity::framework::standard::DispatchError;
use serenity::http;
use serenity::model::id::UserId;
use serenity::prelude::*;

use std::collections::HashSet;
use std::env;
use std::io::prelude::*;
use std::fs::{File, OpenOptions};
use std::path::Path;

struct Handler;
impl EventHandler for Handler {}

fn main() {
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN"), Handler)
        .expect("client err");

    let owners = match http::get_current_application_info() {
        Ok(info) => { let mut set = HashSet::new();
                      set.insert(info.owner.id);
                      set },
        Err(why) => panic!("Couldn't get application info: {:?}", why) };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix(".\\").owners(owners))
            .on_dispatch_error(|_ctx, msg, error| { use DispatchError::*; match error {
                RateLimited(seconds) => {
                    let _ = msg.reply(&format!("RATELIMIT: Try again in {} seconds.", seconds)); },
                OnlyForOwners  => { let _ = msg.reply("You don't own me!"); },
                _ => () }})
            .command("usage", |c| c.cmd(usage))
            .command("init", |c| c.cmd(init))
            .command("mimic", |c| c.cmd(mimic)));

    let _ = client.start(); }

fn does_raw_exist(f: &u64) -> bool { Path::new(&format!("./data/raw/{}", f)).exists() }

command!(usage(_ctx, msg) { let _ = msg.reply("\nUsage:\n.\\mimic @user1 @user2..."); });

command!(init(_ctx, msg) {
    if msg.author.id != UserId(277996711509491714) { let _ = msg.reply("You're not ace, lol!"); }
    else {
        let _ = msg.reply("Initing.");
        let mut m_id = msg.id;
        loop {
            let retriever = GetMessages::default().before(m_id);
            let messages = msg.channel_id.messages(|_| retriever)?;
            match &messages[..] {
                &[] => break,
                msgs => {
                    for m in msgs.iter().filter(|m| !m.author.bot) {
                        let file_name = format!("./data/raw/{}", m.author.id.0);
                        let mut file =
                            if !does_raw_exist(&m.author.id.0) { File::create(&file_name)? }
                            else { OpenOptions::new().append(true).open(&file_name)? };
                        file.write_all(format!("{}\n", m.content).as_bytes())?;
                        file.sync_all()?; }
                    m_id = msgs.last().unwrap().id }}}
        OpenOptions::new().create(true).write(true).open("./data/init").unwrap();
        let _ = msg.reply("Init done."); }});

command!(mimic(_ctx, msg) {
    if !Path::new("./data/init").exists() { let _ = msg.reply("Still working on it! Gibs me time.."); }
    else {
        match &msg.mentions.clone().into_iter().filter(|x| !x.bot).collect::<Vec<_>>()[..] {
            &[] => { let _ = msg.reply("Requires non-bot @mention."); },
            ms  => {
                let mut chain = Chain::of_order(2);
                for m in ms { chain.feed_file(&Path::new(&format!("./data/raw/{}", m.id.0)))?; }

                let mimic_str = ms.iter().fold("".to_string(),
                                               |mut acc, val| { acc.push_str(" ");
                                                                acc.push_str(&val.name);
                                                                acc });
                let mut reply = "".to_string();
                let mut chain_iter = chain.iter();
                let mut num_resp = 0;
                while num_resp < 10 {
                    let line = chain_iter.next();
                    match line {
                        None => { break },
                        Some(line) => {
                            let first_word = &line[0];
                            if first_word.starts_with(";;") ||
                                first_word.starts_with(".") ||
                                first_word.starts_with("!") ||
                                line.len() < 7
                            {}
                            else {
                                for word in &line {
                                    if word.starts_with("<@") {}
                                    else {
                                        reply.push_str(&word);
                                        reply.push_str(" ") }}
                                reply.push_str("\n\n");
                                num_resp += 1 }}}}

                let _ = msg.reply(&format!("Mimicking{}:\n\n{}", mimic_str, reply)); }}}});
