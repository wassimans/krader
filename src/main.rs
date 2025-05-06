use std::{fmt::Display, time::Duration};

use futures::future;
use iced::{
    Color, Element, Subscription, Task, Theme, application,
    time::every,
    widget::{Row, column, scrollable, text},
};
use thiserror::Error;

fn main() -> iced::Result {
    application(Krader::title, Krader::update, Krader::view)
        .subscription(Krader::subscription)
        .theme(Krader::theme)
        .run_with(Krader::new)
}

#[derive(Debug)]
struct WatchItem {
    symbol: String,
    price: Option<f64>,
    last_update: Option<String>,
    error: Option<String>,
}

pub struct Krader {
    watch_list: Vec<WatchItem>,
}

#[derive(Debug)]
enum Message {
    FetchPrices,
    PricesFetched(Result<Vec<(String, f64)>, FetchError>),
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
        let symbols = vec!["XBTUSD", "ETHUSD", "DOTUSD"];
        let watch_list = symbols
            .iter()
            .map(|sym| WatchItem {
                symbol: sym.to_string(),
                price: None,
                last_update: None,
                error: None,
            })
            .collect();
        (
            Krader { watch_list },
            Task::perform(fetch_btc_price(), Message::PricesFetched),
        )
    }

    fn title(&self) -> String {
        "🦑 Krader".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchPrices => Task::perform(fetch_btc_price(), Message::PricesFetched),
            Message::PricesFetched(Ok(prices)) => {
                self.watch_list = prices
                    .iter()
                    .map(|watch_item| WatchItem {
                        symbol: watch_item.0.clone(),
                        price: Some(watch_item.1),
                        last_update: Some(chrono::Utc::now().to_rfc3339()),
                        error: None,
                    })
                    .collect();
                Task::none()
            }

            Message::PricesFetched(Err(err)) => {
                // In case of a global error, store it in each WatchItem
                for item in &mut self.watch_list {
                    item.error = Some(err.to_string());
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let tokens_column = column(self.watch_list.iter().map(|item| {
            Row::new()
                .spacing(20)
                .push(text(&item.symbol).size(24))
                .push(
                    text(
                        item.price
                            .map(|p| format!("{:.2}", p))
                            .unwrap_or_else(|| "-".into()),
                    )
                    .size(24),
                )
                .push(text(item.last_update.clone().unwrap_or_else(|| "--".into())).size(16))
                .push(
                    text(item.error.clone().unwrap_or_default())
                        .color(Color::from_rgb(1.0, 0.0, 0.0))
                        .size(16),
                )
                .into()
        }))
        .spacing(10);

        scrollable(tokens_column).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Send Message::FetchPrice every 5 seconds
        every(Duration::from_secs(5)).map(|_| Message::FetchPrices)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn fetch_btc_price() -> Result<Vec<(String, f64)>, FetchError> {
    let symbols = vec!["XXBTZUSD", "XETHZUSD", "DOTUSD"];
    let mut tasks = Vec::new();
    for &s in &symbols {
        let url = format!("https://api.kraken.com/0/public/Ticker?pair={}", s);
        tasks.push(async move {
            let resp: serde_json::Value = reqwest::get(url).await?.json().await?;
            let price_str = resp["result"][s]["c"][0]
                .as_str()
                .ok_or(FetchError::MissingField)?;

            let price = price_str.parse::<f64>()?;
            Ok((s.into(), price))
        });
    }

    // Run all fetch tasks concurrently
    let results: Vec<Result<(String, f64), FetchError>> = future::join_all(tasks).await;

    // Partition successes and errors
    let mut prices = Vec::new();
    for res in results {
        prices.push(res?);
    }

    Ok(prices)
}
