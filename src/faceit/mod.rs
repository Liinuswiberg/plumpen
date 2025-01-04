use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::{Deserialize};
use shuttle_runtime::__internals::serde_json;

pub struct Faceit {
    token: String
}

#[derive(Deserialize, Debug)]
struct GameInfo {
    skill_level: u32,
    faceit_elo: u32,
}

#[derive(Deserialize)]
struct Player {
    player_id: String,
    nickname: String,
    games: serde_json::Map<String, GameInfo>,
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

    pub async fn get_faceit_user_by_nickname(&self, username: String) -> Result<(Player), Box<dyn std::error::Error>> {
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
            eprintln!("Error: {:?}", response.status());
        }

    }

}
