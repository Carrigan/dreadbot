mod goldfish;
extern crate regex;

mod deck;
mod card;
mod scryfall;

use card::{Cents};
use goldfish::{retrieve_deck};
use deck::Deck;
use regex::Regex;
use std::env;
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

const MAINDECK_LIMIT: Cents = Cents(20_00);
const SIDEBOARD_LIMIT: Cents = Cents(5_00);

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

    deck.update_pricing(scryfall_resp.data);
    Some(deck)
}

fn respond(ctx: Context, msg: &Message, response: String) {
    if let Err(why) = msg.channel_id.say(&ctx.http, response) {
        println!("Error sending response: {:?}", why);
    }
}

fn respond_to_deck(ctx: Context, msg: &Message, deck: &Deck) {
    let maindeck_price = deck.mainboard_pricing();
    let sideboard_price = deck.sideboard_pricing();
    let formatted_maindeck = maindeck_price.format();
    let formatted_sideboard= sideboard_price.format();

    let maindeck_over = deck.mainboard_pricing().0 <= MAINDECK_LIMIT.0;
    let sideboard_over = deck.sideboard_pricing().0 <= SIDEBOARD_LIMIT.0;
    let response = match (maindeck_over, sideboard_over) {
        (true, true) =>
            format!(
                ":white_check_mark: Deck accepted!\nMaindeck price: {}\nSideboard price: {}",
                formatted_maindeck, formatted_sideboard
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

    respond(ctx, &msg, response);
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        let regex =
            Regex::new(r"^!dreadbot https://www\.mtggoldfish\.com/deck/(\d*).*$")
            .unwrap();

        let captures = match regex.captures(&msg.content) {
            Some(c) => c,
            None => return
        };

        let id = match captures.get(1) {
            Some(c) => c.as_str(),
            None => return
        };

        let deck = match fetch_deck(id) {
            Some(d) => d,
            None => {
                let response = format!("Decklist with id {:?} is not accessible or private.", id);
                respond(ctx, &msg, response);
                return;
            }
        };


        respond_to_deck(ctx, &msg, &deck);
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
