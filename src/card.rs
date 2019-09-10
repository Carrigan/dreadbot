#[derive(Debug)]
pub struct Card {
  pub quantity: u8,
  pub name: String
}

impl Card {
  pub fn from_goldfish_line(line: &str) -> Option<Self> {
    if line.is_empty() { return None }

    let mut splitter = line.splitn(2, " ");
    let quantity_string = splitter.next().unwrap();
    let name_string = splitter.next().unwrap();
    let quantity_parsed = quantity_string.parse::<u8>();

    match quantity_parsed {
      Ok(quantity) => Some(Card { quantity: quantity, name: String::from(name_string) }),
      Err(_) => None
    }
  }
}

#[test]
fn test_card_creation() {
  let card = Card::from_goldfish_line("4 Winding Constrictor").unwrap();
  assert_eq!(card.name, "Winding Constrictor");
  assert_eq!(card.quantity, 4);
}

#[test]
fn test_empty_card() {
  let card = Card::from_goldfish_line("");
  assert_eq!(card.is_none(), true);
}
