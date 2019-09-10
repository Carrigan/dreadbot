mod goldfish;
mod deck;
mod card;
mod scryfall;

use goldfish::{retrieve_deck};
use deck::Deck;

fn main() {
    let id = String::from("2248505");
    let resp = retrieve_deck(&id).unwrap();
    let deck = Deck::from_goldfish_block(id, resp);

    let scryfall_resp = scryfall::request_pricing(&deck).unwrap();
    for item in scryfall_resp.data {
        println!("{:?} costs {:?}", item.name, item.prices.usd.as_ref());
    }
}
