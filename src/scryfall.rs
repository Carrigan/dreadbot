extern crate serde_derive;
use serde::{Serialize, Deserialize};
use super::card::Card;
use super::deck::Deck;

#[derive(Serialize, Debug)]
struct ScryfallRequest {
  identifiers: Vec<NameIdentifier>
}

#[derive(Serialize, Debug)]
struct NameIdentifier {
  name: String
}

#[derive(Deserialize, Debug)]
pub struct ScryfallResponse {
  pub data: Vec<ScryfallData>
}

#[derive(Deserialize, Debug)]
pub struct ScryfallData {
  pub name: String,
  pub prices: ScryfallPrices
}

#[derive(Deserialize, Debug)]
pub struct ScryfallPrices {
  pub usd: Option<String>
}

fn card_to_name_identifier(card: &Card) -> NameIdentifier {
  NameIdentifier { name: card.name.clone() }
}

pub fn request_pricing(deck: &Deck) -> Result<ScryfallResponse, Box<dyn std::error::Error>> {
  let uri = "https://api.scryfall.com/cards/collection";
  let mut identifiers: Vec<NameIdentifier> = Vec::new();

  for card in deck.cards() {
    identifiers.push(card_to_name_identifier(card));
  }

  let request_body = ScryfallRequest { identifiers: identifiers };
  let mut response = reqwest::Client::new()
    .post(uri)
    .json(&request_body)
    .send()?;

  let json = response.json()?;

  Ok(json)
}

#[test]
fn test_api_call() {
    let zombie_hunt = "4 Treasure Hunt\r\n4 Zombie Infestation\r\n26 Island\r\n26 Swamp";
    let deck = Deck::from_goldfish_block(String::from("10108"), String::from(zombie_hunt));

    let scryfall_resp = request_pricing(&deck).unwrap();
    for item in &scryfall_resp.data {
        println!("{:?} costs {:?}", item.name, item.prices.usd.as_ref());
    }

    assert_eq!(scryfall_resp.data.get(0).is_some(), true);
}
