#[derive(Debug)]
pub struct Card {
  pub quantity: u8,
  pub name: String
}

impl Card {
  pub fn new(quantity: u8, name: String) -> Self {
    Self{
      quantity: quantity,
      name: name
    }
  }

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
