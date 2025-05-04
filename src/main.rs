use std::time::Duration;

use iced::{
    Color, Element, Subscription, Task, Theme, application,
    time::every,
    widget::{Column, text},
};
use thiserror::Error;

fn main() -> iced::Result {
    application(Krader::title, Krader::update, Krader::view)
        .subscription(Krader::subscription)
        .theme(Krader::theme)
        .run_with(Krader::new)
}

pub struct Krader {
    btc_price: Option<f64>,
    last_update: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug)]
enum Message {
    FetchPrice,
    PriceFetched(Result<f64, FetchError>),
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Price field missing in response")]
    MissingField,

    #[error("Failed to parse price string: {0}")]
    Parse(#[from] std::num::ParseFloatError),
}

impl Krader {
    fn new() -> (Self, Task<Message>) {
        (
            Krader {
                btc_price: None,
                last_update: None,
                last_error: None,
            },
            Task::perform(fetch_btc_price(), Message::PriceFetched),
        )
    }

    fn title(&self) -> String {
        "🦑 Krader".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchPrice => Task::perform(fetch_btc_price(), Message::PriceFetched),
            Message::PriceFetched(Ok(price)) => {
                self.btc_price = Some(price);
                self.last_update = Some(chrono::Utc::now().to_rfc3339());
                Task::none()
            }

            Message::PriceFetched(Err(err)) => {
                // Handle the error: e.g. store it or log it
                self.last_error = Some(err.to_string());
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let price_text = if let Some(p) = self.btc_price {
            format!("BTC/USD: ${:.2}", p)
        } else {
            "Fetching price..".into()
        };

        let update_text = if let Some(ts) = &self.last_update {
            format!("Last update: {}", ts)
        } else {
            "".into()
        };

        let error_text = if let Some(e) = &self.last_error {
            format!("Error: {}", e)
        } else {
            "".into()
        };

        Column::new()
            .push(text(price_text).size(40))
            .push(text(update_text).size(16))
            .push(
                text(error_text)
                    .color(Color::from_rgb(1.0, 0.0, 0.0))
                    .size(16),
            )
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Send Message::FetchPrice every 5 seconds
        every(Duration::from_secs(5)).map(|_| Message::FetchPrice)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn fetch_btc_price() -> Result<f64, FetchError> {
    let url = "https://api.kraken.com/0/public/Ticker?pair=XBTUSD";
    let resp: serde_json::Value = reqwest::get(url).await?.json().await?;

    let price_str = resp["result"]["XXBTZUSD"]["c"][0]
        .as_str()
        .ok_or(FetchError::MissingField)?;

    let price = price_str.parse::<f64>()?;
    Ok(price)
}
