use super::card::{Card};

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
