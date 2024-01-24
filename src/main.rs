extern crate markov;
extern crate serenity;

use std::env;
use std::path::Path;

use markov::Chain;
use serenity::async_trait;
use serenity::builder::GetMessages;
use serenity::model::id::ChannelId;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, Configuration, CommandResult};
use serenity::prelude::*;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

#[group]
#[commands(init, mimic)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
  let framework = StandardFramework::new().group(&GENERAL_GROUP);
  framework.configure(Configuration::new().prefix(".\\"));
  let token = env::var("DISCORD_TOKEN").expect("token");
  let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
  let mut client = Client::builder(token, intents)
    .event_handler(Handler)
    .framework(framework)
    .await
    .expect("Error creating client");

  if let Err(why) = client.start().await {
    println!("An error occurred while running the client: {:?}", why)
  }
}

fn does_raw_exist(f: &u64) -> bool { Path::new(&format!("./data/raw/{}", f)).exists() }

#[command]
async fn init(ctx: &Context, msg: &Message) -> CommandResult {
  if msg.author.id != 377667908098064384 { msg.reply(ctx, "You're not Ace, lol!").await?; }
  else {
    msg.reply(ctx, "Initing.").await?;
    let channel_id = ChannelId::new(1013954641832185908);

    let mut m_id = msg.id;
    loop {
      let retriever = GetMessages::default().before(m_id);
      let messages = channel_id.messages(&ctx.http, retriever).await?;
      msg.reply(ctx, format!("{} messages.", messages.len())).await?;
      match &messages[..] {
        &[] => break,
        msgs => {
          for m in msgs.iter().filter(|m| !m.author.bot) {
            msg.reply(ctx, format!("<@&{}>: {}", m.author.name, m.content)).await?;
            let file_name = format!("./data/raw/{}", m.author.id);
            println!("Writing to {}", file_name);
            let mut file =
                if !does_raw_exist(&m.author.id.get()) { File::create(&file_name).await? }
                else { OpenOptions::new().append(true).open(&file_name).await? };
            let content = m.content.split(" ").filter(|x| !(x.starts_with("http://") || x.starts_with("https://"))).collect::<Vec<_>>().join(" ");
            file.write_all(format!("{}\n", content).as_bytes()).await?;
            file.sync_all().await?; }
          m_id = msgs.last().unwrap().id } } }
    OpenOptions::new().create(true).write(true).open("./data/init").await?;
    msg.reply(ctx, "Done.").await?; }
  Ok(()) }

#[command]
async fn mimic(ctx: &Context, msg: &Message) -> CommandResult {
  if !Path::new("./data/init").exists() { msg.reply(ctx, "Still working on it! Gibs me time..").await?; }
  else {
    match &msg.mentions.clone().into_iter().filter(|x| !x.bot).collect::<Vec<_>>()[..] {
      &[] => { msg.reply(ctx, "Requires non-bot @mention.").await?; },
      ms  => {
        let mut chain : Chain<String> = Chain::of_order(1);
        for m in ms { chain.feed_file(&Path::new(&format!("./data/raw/{}", m.id)))?; }
        msg.reply(ctx, chain.generate_str()).await?; } } }
  Ok(()) }