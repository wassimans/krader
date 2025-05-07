use std::{fmt::Display, time::Duration};

use futures::future;
use iced::{
    Color, Element, Subscription, Task, Theme, application,
    time::every,
    widget::{Column, Row, column, container, scrollable, text},
};
use thiserror::Error;

fn main() -> iced::Result {
    application(Krader::title, Krader::update, Krader::view)
        .subscription(Krader::subscription)
        .theme(Krader::theme)
        .run_with(Krader::new)
}

#[derive(Debug)]
struct OrderBook {
    pair: String,
    bids: Vec<(f64, f64)>, // (price, size)
    asks: Vec<(f64, f64)>,
    last_error: Option<String>,
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
    order_book: OrderBook,
}

#[derive(Debug)]
enum Message {
    FetchPrices,
    PricesFetched(Result<Vec<(String, f64)>, FetchError>),
    FetchOrderBook,
    OrderBookFetched(Result<OrderBook, FetchError>),
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
        let order_book = OrderBook {
            pair: "XBTUSD".into(),
            bids: vec![],
            asks: vec![],
            last_error: None,
        };
        (
            Krader {
                watch_list,
                order_book,
            },
            Task::perform(fetch_all_price(), Message::PricesFetched),
        )
    }

    fn title(&self) -> String {
        "🦑 Krader".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchPrices => Task::perform(fetch_all_price(), Message::PricesFetched),
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
            Message::FetchOrderBook => {
                let pair = self.order_book.pair.clone();
                Task::perform(fetch_order_book(pair), Message::OrderBookFetched)
            }

            Message::OrderBookFetched(Ok(book)) => {
                self.order_book = book;
                Task::none()
            }
            Message::OrderBookFetched(Err(err)) => {
                self.order_book.last_error = Some(err.to_string());
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Heatmap column
        let mut heatmap = Column::new().spacing(2);
        let max_size = self
            .order_book
            .bids
            .iter()
            .chain(self.order_book.asks.iter())
            .map(|(_, size)| *size)
            .fold(0.0, f64::max);

        for (price, size) in &self.order_book.bids {
            let width = (size / max_size * 200.0).max(5.0);
            heatmap = heatmap.push(
                container(text(format!("{:.2}", price)))
                    .width(width)
                    .style(HeatmapBar::Bid),
            );
        }

        for (price, size) in &self.order_book.asks {
            let width = (size / max_size * 200.0).max(5.0);
            heatmap = heatmap.push(
                container(text(format!("{:.2}", price)))
                    .width(width)
                    .style(HeatmapBar::Ask),
            );
        }

        // Right: numeric table
        let mut table = Column::new().spacing(2);
        table = table.push(
            Row::new()
                .spacing(20)
                .push(text("Price"))
                .push(text("Size"))
                .push(text("Total")),
        );
        for (price, size) in &self.order_book.bids {
            let total = price * size;
            table = table.push(
                Row::new()
                    .spacing(20)
                    .push(text(format!("{:.2}", price)).size(16))
                    .push(text(format!("{:.4}", size)).size(16))
                    .push(text(format!("{:.2}", total)).size(16)),
            );
        }
        for (price, size) in &self.order_book.asks {
            let total = price * size;
            table = table.push(
                Row::new()
                    .spacing(20)
                    .push(text(format!("{:.2}", price)).size(16))
                    .push(text(format!("{:.4}", size)).size(16))
                    .push(text(format!("{:.2}", total)).size(16)),
            );
        }

        // Assemble side by side
        Row::new()
            .spacing(40)
            .push(heatmap)
            .push(container(table).width(Length::Fill))
            .into()

        // let tokens_column = column(self.watch_list.iter().map(|item| {
        //     Row::new()
        //         .spacing(20)
        //         .push(text(&item.symbol).size(24))
        //         .push(
        //             text(
        //                 item.price
        //                     .map(|p| format!("{:.2}", p))
        //                     .unwrap_or_else(|| "-".into()),
        //             )
        //             .size(24),
        //         )
        //         .push(text(item.last_update.clone().unwrap_or_else(|| "--".into())).size(16))
        //         .push(
        //             text(item.error.clone().unwrap_or_default())
        //                 .color(Color::from_rgb(1.0, 0.0, 0.0))
        //                 .size(16),
        //         )
        //         .into()
        // }))
        // .spacing(10);

        // scrollable(tokens_column).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let prices = every(Duration::from_secs(5)).map(|_| Message::FetchPrices);
        let book = every(Duration::from_secs(5)).map(|_| Message::FetchOrderBook);

        Subscription::batch(vec![prices, book])
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn fetch_all_price() -> Result<Vec<(String, f64)>, FetchError> {
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

async fn fetch_order_book(pair: String) -> Result<OrderBook, FetchError> {
    let url = format!(
        "https://api.kraken.com/0/public/Depth?pair={}&count=10",
        pair
    );

    let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;
    let data = &resp["result"][&pair];

    // Parse bids and asks arrays
    let parse_side = |side: &serde_json::Value| {
        side.as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.get(0)?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect::<Vec<_>>()
    };

    let bids = parse_side(&data["bids"]);
    let asks = parse_side(&data["asks"]);

    Ok(OrderBook {
        pair,
        bids,
        asks,
        last_error: None,
    })
}
