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
