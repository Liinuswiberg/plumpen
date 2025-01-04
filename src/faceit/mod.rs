use anyhow::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::Deserialize;

pub struct Faceit {
    token: String
}

#[derive(Deserialize)]
pub struct Player {
    pub player_id: String,
    pub nickname: String,
    games: serde_json::Map<String, serde_json::Value>
}

impl Faceit {
    pub fn new(token: String) -> Self {
        Self {
            token
        }
    }

    pub async fn get_faceit_user_by_id() {
        println!("Hello from my_module!");
    }

    pub async fn get_faceit_user_by_nickname(&self, username: String) -> Result<Player, Error> {
        let url = format!("https://open.faceit.com/data/v4/players?nickname={}&game=cs2", username);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token))?);

        let client = Client::new();

        let response = client.get(url)
            .headers(headers)
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;

            let player: Player = serde_json::from_str(&body)?;

            Ok(player)
        } else {
            Err(anyhow::anyhow!("Failed to get faceit user!"))
        }

    }

}
