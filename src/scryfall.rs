extern crate serde_derive;
use serde::{Deserialize};
use super::card::{Card, Cents};
use super::deck::Deck;

#[derive(Deserialize, Debug)]
pub struct ScryfallResponse {
  pub data: Vec<ScryfallData>,
  pub next_link: Option<String>
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

#[derive(Debug)]
pub struct PricingSource {
  pub name: String,
  pub price: Cents
}

const BASIC_LAND_NAMES: &'static [&'static str] = &[
  "Swamp",
  "Island",
  "Forest",
  "Mountain",
  "Plains"
];

fn format_scryfall_param(card: &Card) -> String {
  format!("!\"{}\"", card.name)
}

fn reduce_pricing(entries: Vec<ScryfallData>) -> Vec<PricingSource> {
  let mut prices: Vec<PricingSource> = Vec::new();

  for entry in entries {
    let str_price = match &entry.prices.usd {
      Some(price) => price,
      None => continue
    };

    let price = match str_price.parse::<f32>() {
      Ok(p32) => (p32 * 100f32) as Cents,
      _ => continue
    };

    let previous_entry = prices.iter_mut().find(|ps| ps.name == entry.name);

    // If it exists, update if the new price is lower
    if let Some(previous_price) = previous_entry {
      if price < previous_price.price {
        previous_price.price = price;
      }

    // Otherwise add it
    } else {
      prices.push(PricingSource { name: entry.name, price: price });
    }
  }

  prices
}

pub fn request_pricing(deck: &Deck) -> Result<Vec<PricingSource>, Box<dyn std::error::Error>> {
  let mut name_params = String::new();
  let mut first_flag = true;
  for card in deck.cards() {
    // Advance the first flag if true
    let is_first = first_flag;
    first_flag = false;

    // If it is a basic, do not add it to the list. This returns hundreds of cards each
    if let Some(_) = BASIC_LAND_NAMES.iter().find(|name| *name == &card.name) {
      continue;
    }

    // Add to it
    if !is_first { name_params += " OR "; }
    name_params += &format_scryfall_param(card)
  }

  // Start a list of ScryfallData in case there are multiple requests
  let mut data: Vec<ScryfallData> = Vec::new();

  // Build the initial query
  let query =
    format!("https://api.scryfall.com/cards/search?unique=prints&q=-is:digital and ({})", name_params)
      .replace(" ", "%20");

  // Send it and merge
  let mut response: ScryfallResponse =
    reqwest::Client::new()
      .get(&query)
      .send()?
      .json()?;

  data.append(&mut response.data);

  // Consume until there is no more
  while let Some(next_url) = response.next_link {
    response =
      reqwest::Client::new()
        .get(&next_url)
        .send()?
        .json()?;

    data.append(&mut response.data);
  }

  Ok(reduce_pricing(data))
}

#[test]
fn test_api_call() {
    let zombie_hunt = "4 Treasure Hunt\r\n4 Zombie Infestation\r\n26 Island\r\n26 Swamp";
    let deck = Deck::from_goldfish_block(String::from("10108"), String::from(zombie_hunt));

    let scryfall_resp = request_pricing(&deck).unwrap();
    for item in &scryfall_resp {
        println!("{:?} costs {:?}", item.name, item.price);
    }

    assert_eq!(scryfall_resp.get(0).is_some(), true);
}

#[test]
fn test_request_pricing() {
  let mut scryfall_mock: Vec<ScryfallData> = Vec::new();
  scryfall_mock.push(ScryfallData{
    name: String::from("Island"),
    prices: ScryfallPrices { usd: Some(String::from("1.00")) }});
  scryfall_mock.push(ScryfallData{
    name: String::from("Island"),
    prices: ScryfallPrices { usd: Some(String::from("0.50")) }});
  scryfall_mock.push(ScryfallData{
    name: String::from("Island"),
    prices: ScryfallPrices { usd: Some(String::from("2.00")) }});

  let reduced_prices = reduce_pricing(scryfall_mock);
  assert_eq!(reduced_prices.len(), 1);
  assert_eq!(reduced_prices.get(0).unwrap().name, "Island");
  assert_eq!(reduced_prices.get(0).unwrap().price, 50 as Cents);
}
