extern crate markov;
extern crate serenity;

use std::collections::HashSet;
use std::fs::read_to_string;
use std::env;
use std::path::Path;

use markov::Chain;
use rand::Rng;
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
impl EventHandler for Handler {
  async fn message(&self, ctx: Context, msg: Message) {
    if msg.mentions_user_id(1195538057823268914) {
      let _ = add_line(msg.author.id.get(), msg.content.clone()).await;
      let _ = msg.reply(ctx, mimic_impl(vec![321132914576457728])).await; } } }

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
    println!("An error occurred while running the client: {:?}", why) } }

fn does_raw_exist(f: &u64) -> bool { Path::new(&format!("./data/raw/{}", f)).exists() }

async fn add_line(u: u64, l: String) -> Result<(), std::io::Error> {
  let file_name = format!("./data/raw/{}", u);
  let mut file =
      if !does_raw_exist(&u) { File::create(&file_name).await? }
      else { OpenOptions::new().append(true).open(&file_name).await? };
  let content =
    l.split(" ")
      .filter(|x| !(x.starts_with("http://") || x.starts_with("https://")))
      .collect::<Vec<_>>()
      .join(" ");
  file.write_all(format!("{}\n", content).as_bytes()).await?;
  file.sync_all().await?;
  Ok(()) }

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
      match &messages[..] {
        &[] => break,
        msgs => {
          for m in msgs.iter().filter(|m| !m.author.bot) {
            add_line(m.author.id.get(), m.content.clone()).await?; }
          m_id = msgs.last().unwrap().id } } }
    OpenOptions::new().create(true).write(true).open("./data/init").await?;
    msg.reply(ctx, "Done.").await?; }
  Ok(()) }

fn mimic_impl(ms: Vec<u64>) -> String {
  let mut chain : Chain<String> = Chain::of_order(1);
  let mut set = HashSet::<String>::new();
  for m in ms {
    let path_str = format!("./data/raw/{}", m);
    { let _ = chain.feed_file(Path::new(&path_str)); }
    for line in read_to_string(Path::new(&path_str)).unwrap().lines() {
      set.insert(line.to_string()); } }

  let mut ret = String::new();
  let mut rng = rand::thread_rng();
  let min_msg_len = rng.gen_range(10..=25);
  while ret.split(" ").count() < min_msg_len {
    let mimic = chain.generate_str();
    if !set.contains(&mimic) {
      if ret.len() != 0 { ret.push_str(". "); }
      ret.push_str(&mimic) } }
  ret }

#[command]
async fn mimic(ctx: &Context, msg: &Message) -> CommandResult {
  if !Path::new("./data/init").exists() { msg.reply(ctx, "Still working on it! Gibs me time..").await?; }
  else {
    match &msg.mentions.clone().into_iter().filter(|x| !x.bot).collect::<Vec<_>>()[..] {
      &[] => { msg.reply(ctx, "Requires non-bot @mention.").await?; },
      ms  => {
        let rsp = mimic_impl(ms.iter().map(|u| u.id.get()).collect::<Vec<_>>());
        msg.reply(ctx, rsp).await?; } } }
  Ok(()) }