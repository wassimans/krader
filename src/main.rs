use std::time::Duration;

use iced::{
    Element, Length, Renderer, Subscription, Task, Theme, application,
    time::every,
    widget::{column, container, responsive, scrollable, text},
};

use iced_table::table;
use serde::{Deserialize, Serialize};
use thiserror::Error;

fn main() -> iced::Result {
    application(Krader::title, Krader::update, Krader::view)
        .subscription(Krader::subscription)
        .theme(Krader::theme)
        .run_with(Krader::new)
}

pub struct Krader {
    columns: Vec<WatchlistColumn>,
    watch_list: Vec<WatchItem>,
    header: scrollable::Id,
    body: scrollable::Id,
    footer: scrollable::Id,
    resize_columns_enabled: bool,
    footer_enabled: bool,
    min_width_enabled: bool,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    SyncHeader(scrollable::AbsoluteOffset),
    Resizing(usize, f32),
    Resized,
    FetchData,
    DataFetched(Result<Vec<WatchItem>, String>),
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
            Self {
                columns: vec![
                    WatchlistColumn::new(ColumnKind::Pair),
                    WatchlistColumn::new(ColumnKind::MarkPrice),
                    WatchlistColumn::new(ColumnKind::Vol24h),
                    WatchlistColumn::new(ColumnKind::VolumeQuote),
                ],
                watch_list: vec![],
                header: scrollable::Id::unique(),
                body: scrollable::Id::unique(),
                footer: scrollable::Id::unique(),
                resize_columns_enabled: true,
                footer_enabled: true,
                min_width_enabled: true,
                theme: Theme::Light,
            },
            Task::perform(
                async {
                    fetch_data().await.map_err(|e| e.to_string())
                },
                Message::DataFetched
            ),
        )
    }

    fn title(&self) -> String {
        "🦑 Krader".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SyncHeader(offset) => {
                return Task::batch(vec![
                    scrollable::scroll_to(self.header.clone(), offset),
                    scrollable::scroll_to(self.footer.clone(), offset),
                ]);
            }
            Message::Resizing(index, offset) => {
                if let Some(column) = self.columns.get_mut(index) {
                    column.resize_offset = Some(offset);
                }
                Task::none()
            }
            Message::Resized => {
                self.columns.iter_mut().for_each(|column| {
                    if let Some(offset) = column.resize_offset.take() {
                        column.width += offset;
                    }
                });
                Task::none()
            }
            Message::FetchData => Task::perform(
                async {
                    fetch_data().await.map_err(|e| e.to_string())
                },
                Message::DataFetched
            ),
            Message::DataFetched(Ok(watch_list)) => {
                self.watch_list = watch_list;
                Task::none()
            }
            Message::DataFetched(Err(e)) => {
                eprintln!("{e}");
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let table = responsive(|size| {
            let mut table = table(
                self.header.clone(),
                self.body.clone(),
                &self.columns,
                &self.watch_list,
                Message::SyncHeader,
            );

            if self.resize_columns_enabled {
                table = table.on_column_resize(Message::Resizing, Message::Resized);
            }
            if self.footer_enabled {
                table = table.footer(self.footer.clone());
            }
            if self.min_width_enabled {
                table = table.min_width(size.width);
            }

            table.into()
        });

        let content = column![table,].spacing(6);

        container(container(content).width(Length::Fill).height(Length::Fill))
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let prices = every(Duration::from_secs(5)).map(|_| Message::FetchData);

        Subscription::batch(vec![prices])
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn fetch_data() -> Result<Vec<WatchItem>, FetchError> {
    let url = format!("https://futures.kraken.com/derivatives/api/v3/tickers");
    let resp: TickersResponse = reqwest::get(url).await?.json().await?;

    Ok(resp.tickers)
}

pub(crate) struct WatchlistColumn {
    kind: ColumnKind,
    width: f32,
    resize_offset: Option<f32>,
    enabled: bool,
}

impl WatchlistColumn {
    fn new(kind: ColumnKind) -> Self {
        let width = match kind {
            ColumnKind::Pair => 155.0,
            ColumnKind::MarkPrice => 155.0,
            ColumnKind::Vol24h => 100.0,
            ColumnKind::VolumeQuote => 100.0,
            _ => 50.0,
        };

        Self {
            kind,
            width,
            resize_offset: None,
            enabled: true,
        }
    }
}

enum ColumnKind {
    Symbol,
    Last,
    LastTime,
    Tag,
    Pair,
    MarkPrice,
    Bid,
    BidSize,
    Ask,
    AskSize,
    Vol24h,
    VolumeQuote,
    OpenInterest,
    Open24h,
    High24h,
    Low24h,
    LastSize,
    FundingRate,
    FundingRatePrediction,
    Suspended,
    IndexPrice,
    PostOnly,
    Change24h,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WatchItem {
    symbol: Option<String>,
    last: Option<f64>,
    last_time: Option<String>,
    tag: Option<String>,
    pair: Option<String>,
    mark_price: Option<f64>,
    bid: Option<f64>,
    bid_size: Option<f64>,
    ask: Option<f64>,
    ask_size: Option<f64>,
    vol24h: Option<f64>,
    volume_quote: Option<f64>,
    open_interest: Option<f64>,
    open24h: Option<f64>,
    high24h: Option<f64>,
    low24h: Option<f64>,
    last_size: Option<f64>,
    funding_rate: Option<f64>,
    funding_rate_prediction: Option<f64>,
    suspended: Option<bool>,
    index_price: Option<f64>,
    post_only: Option<bool>,
    change24h: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
struct TickersResponse {
    tickers: Vec<WatchItem>,
}

impl<'a> table::Column<'a, Message, Theme, Renderer> for WatchlistColumn {
    type Row = WatchItem;

    fn header(&'a self, _col_index: usize) -> Element<'a, Message> {
        let content = match self.kind {
            ColumnKind::Pair => "MARKET",
            ColumnKind::MarkPrice => "PRICE",
            ColumnKind::Vol24h => "24H%",
            ColumnKind::VolumeQuote => "V.QUOTE",
            ColumnKind::Symbol => "SYMBOL",
            ColumnKind::Last => "LAST",
            ColumnKind::LastTime => "L.TIME",
            ColumnKind::Tag => "TAG",
            ColumnKind::Bid => "BID",
            ColumnKind::BidSize => "B.SIZE",
            ColumnKind::Ask => "ASK",
            ColumnKind::AskSize => "A.SIZE",
            ColumnKind::OpenInterest => "O.INTEREST",
            ColumnKind::Open24h => "O.24H",
            ColumnKind::High24h => "H.24H",
            ColumnKind::Low24h => "L.24H",
            ColumnKind::LastSize => "L.SIZE",
            ColumnKind::FundingRate => "F.RATE",
            ColumnKind::FundingRatePrediction => "F.R.PREDICTION",
            ColumnKind::Suspended => "SUSPENDED",
            ColumnKind::IndexPrice => "I.PRICE",
            ColumnKind::PostOnly => "P.ONLY",
            ColumnKind::Change24h => "C.24H",
        };

        container(text(content)).center_y(24).into()
    }

    fn cell(
        &'a self,
        _col_index: usize,
        row_index: usize,
        row: &'a WatchItem,
    ) -> Element<'a, Message> {
        let content: Element<_> = match self.kind {
            ColumnKind::Symbol => text(row.symbol.clone().unwrap_or("--".to_string())).into(),
            ColumnKind::Last => text(row.last.unwrap_or_default().to_string()).into(),
            ColumnKind::LastTime => text(row.last_time.clone().unwrap_or("--".to_string())).into(),
            ColumnKind::Tag => text(row.tag.clone().clone().unwrap_or("--".to_string())).into(),
            ColumnKind::Pair => text(row.pair.clone().unwrap_or("--".to_string())).into(),
            ColumnKind::MarkPrice => text(row.mark_price.unwrap_or_default().to_string()).into(),
            ColumnKind::Bid => text(row.bid.unwrap_or_default().to_string()).into(),
            ColumnKind::BidSize => text(row.bid_size.unwrap_or_default().to_string()).into(),
            ColumnKind::Ask => text(row.ask.unwrap_or_default().to_string()).into(),
            ColumnKind::AskSize => text(row.ask_size.unwrap_or_default().to_string()).into(),
            ColumnKind::Vol24h => text(row.vol24h.unwrap_or_default().to_string()).into(),
            ColumnKind::VolumeQuote => {
                text(row.volume_quote.unwrap_or_default().to_string()).into()
            }
            ColumnKind::OpenInterest => {
                text(row.open_interest.unwrap_or_default().to_string()).into()
            }
            ColumnKind::Open24h => text(row.open24h.unwrap_or_default().to_string()).into(),
            ColumnKind::High24h => text(row.high24h.unwrap_or_default().to_string()).into(),
            ColumnKind::Low24h => text(row.low24h.unwrap_or_default().to_string()).into(),
            ColumnKind::LastSize => text(row.last_size.unwrap_or_default().to_string()).into(),
            ColumnKind::FundingRate => {
                text(row.funding_rate.unwrap_or_default().to_string()).into()
            }
            ColumnKind::FundingRatePrediction => {
                text(row.funding_rate_prediction.unwrap_or_default().to_string()).into()
            }
            ColumnKind::Suspended => text(row.suspended.unwrap_or_default().to_string()).into(),
            ColumnKind::IndexPrice => text(row.index_price.unwrap_or_default().to_string()).into(),
            ColumnKind::PostOnly => text(row.post_only.unwrap_or_default().to_string()).into(),
            ColumnKind::Change24h => text(row.change24h.unwrap_or_default().to_string()).into(),
        };

        container(content).width(Length::Fill).center_y(32).into()
    }

    fn footer(&'a self, _col_index: usize, rows: &'a [Self::Row]) -> Option<Element<'a, Message>> {
        let content = Element::from(text(format!("Footer text")));
        Some(container(content).center_y(24).into())
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn resize_offset(&self) -> Option<f32> {
        self.resize_offset
    }
}
