use rss::Channel;
use std::io::Read;
use serde::Deserialize;

use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use diesel::Connection;
use diesel::{Queryable, Insertable};

use lettre::Message;
use lettre::Transport;

use ardina::schema::items;

#[derive(Deserialize)]
pub struct Config {
    database: DatabaseConfig,
    email: EmailConfig,
    feed: FeedConfig,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    url: String,
}

#[derive(Deserialize)]
pub struct EmailConfig {
    relay: String,
    username: String,
    password: String,
    from: String,
    subject_prefix: String,
    subscribers: Vec<String>,
}

#[derive(Deserialize)]
pub struct FeedConfig {
    url: String,
}


#[derive(Queryable, Insertable)]
struct Item {
    pub guid: String,
    pub title: Option<String>,
}

fn fetch_feed(config: &FeedConfig) -> Result<Channel, rss::Error> {
    let body = reqwest::blocking::get(&config.url)
        .unwrap()
        .bytes()
        .unwrap();

    Channel::read_from(&body[..])
}

pub fn never_seen_before(item: &rss::Item, connection: &mut diesel::sqlite::SqliteConnection) -> bool {
    use ardina::schema::items::dsl::*;

    let item_guid = item.guid().unwrap().value();

    let count = items.filter(guid.eq(item_guid)).count().get_result::<i64>(connection);

    count.unwrap() == 0
}

pub fn mark_as_seen(item: &rss::Item, connection: &mut diesel::sqlite::SqliteConnection) {
    diesel::insert_into(ardina::schema::items::table).values(Item {
        guid: item.guid().unwrap().value().to_string(),
        title: Some(item.title().unwrap().to_string()),
    }).execute(connection).unwrap();
}

pub fn send_email(item: &rss::Item, config: &EmailConfig) {
    use lettre::{SmtpTransport, transport::smtp::authentication::Credentials};

    let mailer = SmtpTransport::relay(&config.relay)
        .unwrap()
        .credentials(Credentials::new(config.username.clone(), config.password.clone()))
        .build();

    for subscriber in config.subscribers.iter() {
        println!("{}: {}", subscriber, item.title().unwrap());
        let msg = Message::builder()
            .from(config.from.parse().unwrap())
            .to(subscriber.parse().unwrap())
            .subject(format!("{} {}", config.subject_prefix, item.title().unwrap()))
            .multipart(lettre::message::MultiPart::alternative().singlepart(lettre::message::SinglePart::builder().header(lettre::message::header::ContentType::TEXT_HTML).body(item.description().unwrap().to_string())))
            .unwrap();

        mailer.send(&msg).unwrap();
    }
}

pub fn read_config(filename: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let config = toml::from_str(&content)?;
    Ok(config)
}

fn main() {
    let config: Config = read_config("ardina.toml").unwrap();

    let mut database = SqliteConnection::establish(&config.database.url).unwrap();

    let feed = fetch_feed(&config.feed).unwrap();

    for item in feed.items().iter().rev().take(3) {
        if never_seen_before(&item, &mut database) {
            mark_as_seen(&item, &mut database);
            send_email(&item, &config.email);
        }
    }
}
