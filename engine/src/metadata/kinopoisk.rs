// Placeholder Kinopoisk client
// TODO: Implement actual Kinopoisk API integration

use anyhow::Result;

pub struct KinopoiskClient {
    api_key: Option<String>,
}

impl KinopoiskClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    pub async fn search_movie(&self, _title: &str, _year: Option<u32>) -> Result<()> {
        Ok(())
    }

    pub async fn get_movie_details(&self, _kp_id: u32) -> Result<()> {
        Ok(())
    }
}
