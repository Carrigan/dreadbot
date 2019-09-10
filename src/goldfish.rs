pub fn retrieve_deck(id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let uri = format!("https://www.mtggoldfish.com/deck/download/{}", id);
    let resp: String = reqwest::get(&uri)?.text()?;

    Ok(resp)
}
