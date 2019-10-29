mod goldfish;
extern crate regex;

mod deck;
mod card;
mod scryfall;

use card::{Cents, format_cents};
use goldfish::{retrieve_deck};
use deck::Deck;
use regex::Regex;
use std::env;
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use lazy_static::lazy_static;

const MAINDECK_LIMIT: Cents = 20_00;
const SIDEBOARD_LIMIT: Cents = 5_00;
const DREADBOT_PREFIX: &str = r"^\$\$(.*)$";
const HELP_TEXT: &str =
r"
```
Dreadbot is the official pricing method of paper dreadful.

Commands:
$$help         - Display this message
$$verify <url> - Verify a decklist
$$hash <url>   - Check the hash of a decklist
$$info <url>   - Receive an itemized list of prices for a deck.
                 The response is lengthy so try to keep this to PMs.
```
";
lazy_static! {
    static ref PREFIX_REGEX: Regex = Regex::new(DREADBOT_PREFIX).unwrap();
    static ref INFO_REGEX: Regex = Regex::new(r"^info https://www\.mtggoldfish\.com/deck/(\d*).*$").unwrap();
    static ref HASH_REGEX: Regex = Regex::new(r"^hash https://www\.mtggoldfish\.com/deck/(\d*).*$").unwrap();
    static ref VERIFY_REGEX: Regex = Regex::new(r"^verify https://www\.mtggoldfish\.com/deck/(\d*).*$").unwrap();
}

struct Handler;

fn fetch_deck(id: &str) -> Option<Deck> {
    let response = match retrieve_deck(id) {
        Ok(resp) => resp,
        _ => return None
    };

    let mut deck = Deck::from_goldfish_block(String::from(id), response);

    let scryfall_resp = match scryfall::request_pricing(&deck) {
        Ok(resp) => resp,
        _ => return None
    };

    deck.update_pricing(scryfall_resp);
    Some(deck)
}

fn respond(ctx: &Context, msg: &Message, response: &str) -> bool {
    if let Err(why) = msg.channel_id.say(&ctx.http, response) {
        println!("Error sending response: {:?}", why);
    }

    true
}

fn respond_to_deck(ctx: &Context, msg: &Message, deck: &Deck) -> bool {
    let maindeck_price = deck.mainboard_pricing();
    let sideboard_price = deck.sideboard_pricing();
    let formatted_maindeck = format_cents(maindeck_price);
    let formatted_sideboard= format_cents(sideboard_price);

    let maindeck_over = deck.mainboard_pricing() <= MAINDECK_LIMIT;
    let sideboard_over = deck.sideboard_pricing() <= SIDEBOARD_LIMIT;
    let response = match (maindeck_over, sideboard_over) {
        (true, true) =>
            format!(
                ":white_check_mark: Deck accepted!\nDeck hash: {}\nMaindeck price: {}\nSideboard price: {}",
                deck.to_hash(), formatted_maindeck, formatted_sideboard
            ),
        (true, false) =>
            format!(
                ":x: Deck error! Sideboard overpriced.\nMaindeck price: {}\nSideboard price: {}",
                formatted_maindeck, formatted_sideboard
            ),
        (false, true) =>
            format!(
                ":x: Deck error! Maindeck overpriced.\nMaindeck price: {}\nSideboard price: {}",
                formatted_maindeck, formatted_sideboard
            ),
        _ =>
            format!(
                ":x: Deck error! Maindeck and sideboard overpriced.\nMaindeck price: {}\nSideboard price: {}",
                formatted_maindeck, formatted_sideboard
            )
    };

    respond(ctx, &msg, &response)
}

fn retrieve_or_error(ctx: &Context, msg: &Message, regex: &Regex, parsed_message: &str) -> Option<Deck> {
    let captures = match regex.captures(parsed_message) {
        Some(c) => c,
        None => return None
    };

    let id = match captures.get(1) {
        Some(c) => c.as_str(),
        None => return None
    };

    let deck = fetch_deck(id);
    if deck.is_none() {
        let response = format!("Decklist with id {:?} is not accessible or private.", id);
        respond(ctx, &msg, &response);
    }

    deck
}

fn dreadbot_help(ctx: &Context, msg: &Message) -> bool {
    respond(ctx, &msg, HELP_TEXT)
}

fn dreadbot_verify(ctx: &Context, msg: &Message, parsed_message: &str) -> bool {

    if let Some(deck) = retrieve_or_error(&ctx, &msg, &VERIFY_REGEX, parsed_message) {
        return respond_to_deck(ctx, &msg, &deck);
    }

    false
}

fn dreadbot_info(ctx: &Context, msg: &Message, parsed_message: &str) -> bool {

    if let Some(deck) = retrieve_or_error(&ctx, &msg, &INFO_REGEX, parsed_message) {
        return respond(ctx, &msg, &deck.info_string());
    }

    false
}

fn dreadbot_hash(ctx: &Context, msg: &Message, parsed_message: &str) -> bool {

    if let Some(deck) = retrieve_or_error(&ctx, &msg, &HASH_REGEX, parsed_message) {
        return respond(ctx, &msg, &format!("Deck hash: {}", &deck.to_hash()));
    }

    false
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        let regex = Regex::new(DREADBOT_PREFIX).unwrap();

        if let Some(captures) = regex.captures(&msg.content) {
            if let Some(remaining_message) = captures.get(1) {
                if dreadbot_verify(&ctx, &msg, remaining_message.as_str()) { return }
                if dreadbot_info(&ctx, &msg, remaining_message.as_str()) { return }
                if dreadbot_hash(&ctx, &msg, remaining_message.as_str()) { return }

                // Fallback to the help message
                dreadbot_help(&ctx, &msg);
            };
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler)
        .expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
