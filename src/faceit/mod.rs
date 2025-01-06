use std::env;
use anyhow::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::Deserialize;

pub struct Faceit;

#[derive(Deserialize, Debug)]
pub struct Player {
    pub player_id: String,
    pub nickname: String,
    games: serde_json::Map<String, serde_json::Value>,
}

impl Player {

    pub fn get_player_elo(&self) -> Option<String> {
        let Some(cs2_data) = self.games.get("cs2") else {
            return None;
        };
        let Some(cs2_elo) = cs2_data.get("faceit_elo") else {
            return None;
        };
        Some(cs2_elo.to_string())
    }

    pub fn get_player_skill_level(&self) -> Option<usize> {
        let Some(cs2_data) = self.games.get("cs2") else {
            return None;
        };
        let Some(cs2_skill_level) = cs2_data.get("skill_level") else {
            return None;
        };

        match cs2_skill_level.to_string().parse::<usize>() {
            Ok(number) => Some(number),
            Err(e) => None,
        }
    }

}

impl Faceit {

    pub async fn get_faceit_user_by_id(faceit_id: &String) -> Result<Option<Player>, Error> {

        let url = format!("https://open.faceit.com/data/v4/players/{}", faceit_id);

        let results = Self::faceit_api_query(url).await?;

        Ok(results)

    }

    pub async fn get_faceit_user_by_nickname(username: String) -> Result<Option<Player>, Error> {

        let url = format!("https://open.faceit.com/data/v4/players?nickname={}&game=cs2", username);

        let results = Self::faceit_api_query(url).await?;

        Ok(results)

    }

    async fn faceit_api_query(url: String) -> Result<Option<Player>, Error>{

        let token = env::var("FACEIT_TOKEN").expect("Failed to get FACEIT_TOKEN!");

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))?);

        let client = Client::new();

        let response = client.get(url)
            .headers(headers)
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;

            let player: Player = serde_json::from_str(&body)?;

            Ok(Some(player))
        } else if response.status().is_client_error() {
            Ok(None)
        } else {
            Err(anyhow::anyhow!("Failed to get faceit user!"))
        }

    }

}
