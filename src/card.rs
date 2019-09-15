#[derive(Debug, PartialEq)]
pub struct Cents(pub u32);

impl Cents {
  pub fn format(&self) -> String {
    let dollars = self.0 / 100;
    let remainder = self.0 % 100;

    format!("{}.{:02}", dollars, remainder)
  }
}

#[derive(Debug)]
pub struct Card {
  pub quantity: u32,
  pub name: String,
  pub price: Option<Cents>
}

impl Card {
  pub fn from_goldfish_line(line: &str) -> Option<Self> {
    if line.is_empty() { return None }

    let mut splitter = line.splitn(2, " ");
    let quantity_string = splitter.next().unwrap();
    let name_string = splitter.next().unwrap();
    let quantity_parsed = quantity_string.parse::<u32>();

    match quantity_parsed {
      Ok(quantity) => Some(Card { quantity: quantity, name: String::from(name_string), price: None }),
      Err(_) => None
    }
  }

  pub fn info_string(&self) -> String {
    match &self.price {
      Some(amount) => format!(
        "{} {} ({} each, {} total)",
        self.quantity, self.name, amount.format(), Cents(amount.0 * self.quantity).format()
      ),
      None => format!("{} {} (unpriced)", self.quantity, self.name)
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
