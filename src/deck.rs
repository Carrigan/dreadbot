use super::card::{Card, Cents};
use super::scryfall::{ScryfallData};

#[derive(Debug)]
pub struct Deck {
  goldfish_id: String,
  mainboard: Vec<Card>,
  sideboard: Vec<Card>
}

impl Deck {
  pub fn from_goldfish_block(goldfish_id: String, block: String) -> Self {
    let mut mainboard: Vec<Card> = Vec::new();
    let mut sideboard: Vec<Card> = Vec::new();
    let mut sideboard_flag = false;

    for line in block.split("\r\n") {
        match Card::from_goldfish_line(line) {
            Some(card) => {
              if sideboard_flag { sideboard.push(card) } else { mainboard.push(card) }
            },
            None => sideboard_flag = true
        };
    }

    Deck {
      goldfish_id: goldfish_id,
      mainboard: mainboard,
      sideboard: sideboard
    }
  }

  fn update_card_pricing(card: &mut Card, entry: &ScryfallData) {
    if card.name == entry.name {
      if let Some(price) = &entry.prices.usd {
        if let Ok(price_f32) = price.parse::<f32>() {
          card.price = Some(Cents((price_f32 * 100f32) as u32))
        }
      }
    }
  }

  pub fn update_pricing(&mut self, scryfall_entries: Vec<ScryfallData>) {
    for entry in scryfall_entries {
      for card in &mut self.mainboard {
        Self::update_card_pricing(card, &entry);
      }

      for card in &mut self.sideboard {
        Self::update_card_pricing(card, &entry);
      }
    }
  }

  pub fn cards<'a>(&'a self) -> DeckIter<'a> {
    DeckIter { deck: self, index: 0 }
  }

  fn sum_prices(cards: &Vec<Card>) -> Cents {
    let mut total_cents: Cents = Cents(0);

    for card in cards {
      total_cents = match &card.price {
        Some(amount) => Cents(total_cents.0 + card.quantity * amount.0),
        None => total_cents
      };
    }

    total_cents
  }

  pub fn mainboard_pricing(&self) -> Cents {
    Deck::sum_prices(&self.mainboard)
  }

  pub fn sideboard_pricing(&self) -> Cents {
    Deck::sum_prices(&self.sideboard)
  }

  pub fn info_string(&self) -> String {
    let mut info = String::new();

    info += "```\n";

    info += "Mainboard:\n";
    for card in &self.mainboard {
      info += &card.info_string();
      info += "\n";
    }

    info += "\nSideboard:\n";
    for card in &self.sideboard {
      info += &card.info_string();
      info += "\n";
    }

    info += "```";
    info
  }
}

pub struct DeckIter<'a> {
  deck: &'a Deck,
  index: usize
}

impl <'a> Iterator for DeckIter<'a> {
  type Item = &'a Card;

  fn next(&mut self) -> Option<Self::Item> {
    let mainboard_size = self.deck.mainboard.len();
    if self.index < mainboard_size {
      let card = self.deck.mainboard.get(self.index);
      self.index += 1;
      return card;
    }

    let shifted_index = self.index - mainboard_size;
    if shifted_index < self.deck.sideboard.len() {
      let card = self.deck.sideboard.get(shifted_index);
      self.index += 1;
      return card;
    }

    None
  }
}

#[test]
fn test_deck_creation() {
  let deck_text = "4 Treasure Hunt\r\n4 Zombie Infestation\r\n26 Island\r\n26 Swamp\r\n\r\n15 Good Sideboard Card";
  let id = "test id";
  let deck = Deck::from_goldfish_block(String::from(id), String::from(deck_text));

  assert_eq!(deck.goldfish_id, String::from(id));
  assert_eq!(deck.mainboard.len(), 4);
  assert_eq!(deck.mainboard.get(0).unwrap().quantity, 4);
  assert_eq!(deck.mainboard.get(0).unwrap().name, "Treasure Hunt");

  assert_eq!(deck.sideboard.len(), 1);
  assert_eq!(deck.sideboard.get(0).unwrap().quantity, 15);
  assert_eq!(deck.sideboard.get(0).unwrap().name, "Good Sideboard Card");
}

#[test]
fn test_iterator() {
  let deck_text = "10 Island\r\n4 Treasure Hunt\r\n4 Zombie Infestation\r\n\r\n26 Island";
  let id = "test id";
  let deck = Deck::from_goldfish_block(String::from(id), String::from(deck_text));
  let mut deck_iter = deck.cards();

  assert_eq!(deck_iter.next().unwrap().name, "Island");
  assert_eq!(deck_iter.next().unwrap().name, "Treasure Hunt");
  assert_eq!(deck_iter.next().unwrap().name, "Zombie Infestation");
  assert_eq!(deck_iter.next().unwrap().name, "Island");
  assert_eq!(deck_iter.next().is_none(), true);
}

#[test]
fn test_pricing_update() {
  let deck_text = "10 Island\r\n4 Treasure Hunt";
  let id = "test id";
  let mut deck = Deck::from_goldfish_block(String::from(id), String::from(deck_text));
  let mut scryfall_entries: Vec<ScryfallData> = Vec::new();

  scryfall_entries.push(ScryfallData {
    name: String::from("Island"),
    prices: super::scryfall::ScryfallPrices {
      usd: Some(String::from("1.00"))
    }
  });

  deck.update_pricing(scryfall_entries);

  let island = deck.mainboard.get(0).unwrap();
  assert_eq!(island.price, Some(Cents(100)));

  let treasure_hunt = deck.mainboard.get(1).unwrap();
  assert_eq!(treasure_hunt.price, None);
}

#[test]
fn test_mainboard_pricing() {
  let mut cards: Vec<Card> = Vec::new();
  cards.push(Card { quantity: 10, name: String::from("Island"), price: Some(Cents(100)) });
  cards.push(Card { quantity: 1, name: String::from("Island"), price: None });

  let deck = Deck {
    mainboard: cards,
    sideboard: Vec::new(),
    goldfish_id: String::from("test")
  };

  assert_eq!(deck.mainboard_pricing(), Cents(1000));
}

#[test]
fn test_sideboard_pricing() {
  let mut cards: Vec<Card> = Vec::new();
  cards.push(Card { quantity: 10, name: String::from("Island"), price: Some(Cents(100)) });
  cards.push(Card { quantity: 1, name: String::from("Island"), price: None });

  let deck = Deck {
    mainboard: Vec::new(),
    sideboard: cards,
    goldfish_id: String::from("test")
  };

  assert_eq!(deck.sideboard_pricing(), Cents(1000));
}
