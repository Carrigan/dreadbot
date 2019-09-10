mod goldfish;
mod deck;
mod card;

use goldfish::{retrieve_deck};
use deck::Deck;

fn main() {
    let id = String::from("2248505");
    let resp = retrieve_deck(&id).unwrap();
    let deck = Deck::from_goldfish_block(id, resp);

    println!("{:?}", deck);
}
