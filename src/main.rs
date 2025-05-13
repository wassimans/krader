use std::time::Duration;

use iced::{
    application, time::every, widget::{column, container, horizontal_space, responsive, scrollable, text}, Color, Element, Length, Renderer, Subscription, Task, Theme
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
                    WatchlistColumn::new(ColumnKind::Symbol),
                    WatchlistColumn::new(ColumnKind::Last),
                    WatchlistColumn::new(ColumnKind::LastTime),
                    WatchlistColumn::new(ColumnKind::Tag),
                    WatchlistColumn::new(ColumnKind::Bid),
                    WatchlistColumn::new(ColumnKind::BidSize),
                    WatchlistColumn::new(ColumnKind::Ask),
                    WatchlistColumn::new(ColumnKind::AskSize),
                    WatchlistColumn::new(ColumnKind::OpenInterest),
                    WatchlistColumn::new(ColumnKind::Open24h),
                    WatchlistColumn::new(ColumnKind::High24h),
                    WatchlistColumn::new(ColumnKind::Low24h),
                    WatchlistColumn::new(ColumnKind::LastSize),
                    WatchlistColumn::new(ColumnKind::FundingRate),
                    WatchlistColumn::new(ColumnKind::FundingRatePrediction),
                    WatchlistColumn::new(ColumnKind::Suspended),
                    WatchlistColumn::new(ColumnKind::IndexPrice),
                    WatchlistColumn::new(ColumnKind::PostOnly),
                    WatchlistColumn::new(ColumnKind::Change24h),
                ],
                watch_list: vec![],
                header: scrollable::Id::unique(),
                body: scrollable::Id::unique(),
                footer: scrollable::Id::unique(),
                resize_columns_enabled: true,
                footer_enabled: true,
                min_width_enabled: true,
            },
            Task::perform(
                async { fetch_data().await.map_err(|e| e.to_string()) },
                Message::DataFetched,
            ),
        )
    }

    fn title(&self) -> String {
        "🦑 Krader".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SyncHeader(offset) => Task::batch(vec![
                scrollable::scroll_to(self.header.clone(), offset),
                scrollable::scroll_to(self.footer.clone(), offset),
            ]),
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
                async { fetch_data().await.map_err(|e| e.to_string()) },
                Message::DataFetched,
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
        let time_status = iced::widget::Row::new()
            .height(24)
            .padding(5)
            .push(
                text("Last update: ")
                    .size(14)
                    .color(Color::from_rgb(0.0, 1.0, 0.0)),
            )
            .push(
                text(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
                    .size(14)
                    .color(Color::from_rgb(0.0, 1.0, 0.0)),
            );

        let content = column![table, time_status].spacing(6);

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
    let url = "https://futures.kraken.com/derivatives/api/v3/tickers".to_string();
    let resp: TickersResponse = reqwest::get(url).await?.json().await?;

    Ok(resp.tickers)
}

pub(crate) struct WatchlistColumn {
    kind: ColumnKind,
    width: f32,
    resize_offset: Option<f32>,
}

impl WatchlistColumn {
    fn new(kind: ColumnKind) -> Self {
        let width = match kind {
            ColumnKind::Pair => 100.0,
            ColumnKind::MarkPrice => 100.0,
            ColumnKind::Vol24h => 100.0,
            ColumnKind::VolumeQuote => 100.0,
            ColumnKind::Symbol => 100.0,
            ColumnKind::Last => 100.0,
            ColumnKind::LastTime => 100.0,
            ColumnKind::Tag => 100.0,
            ColumnKind::Bid => 100.0,
            ColumnKind::BidSize => 100.0,
            ColumnKind::Ask => 100.0,
            ColumnKind::AskSize => 100.0,
            ColumnKind::OpenInterest => 100.0,
            ColumnKind::Open24h => 100.0,
            ColumnKind::High24h => 100.0,
            ColumnKind::Low24h => 100.0,
            ColumnKind::LastSize => 100.0,
            ColumnKind::FundingRate => 100.0,
            ColumnKind::FundingRatePrediction => 100.0,
            ColumnKind::Suspended => 100.0,
            ColumnKind::IndexPrice => 100.0,
            ColumnKind::PostOnly => 100.0,
            ColumnKind::Change24h => 100.0,
        };

        Self {
            kind,
            width,
            resize_offset: None,
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
#[serde(rename_all = "camelCase")]
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
        _row_index: usize,
        row: &'a WatchItem,
    ) -> Element<'a, Message> {
        let content: Element<_> = match self.kind {
            ColumnKind::Symbol => text(row.symbol.clone().unwrap_or("N/A".to_string())).into(),
            ColumnKind::Last => text(row.last.unwrap_or_default().to_string()).into(),
            ColumnKind::LastTime => text(row.last_time.clone().unwrap_or("N/A".to_string())).into(),
            ColumnKind::Tag => text(row.tag.clone().clone().unwrap_or("N/A".to_string())).into(),
            ColumnKind::Pair => text(row.pair.clone().unwrap_or("N/A".to_string())).into(),
            ColumnKind::MarkPrice => text(
                row.mark_price
                    .map_or("N/A".to_string(), |v| format!("{}", v)),
            )
            .into(),
            ColumnKind::Bid => text(row.bid.map_or("N/A".to_string(), |v| format!("{}", v))).into(),
            ColumnKind::BidSize => {
                text(row.bid_size.map_or("N/A".to_string(), |v| format!("{}", v))).into()
            }
            ColumnKind::Ask => text(row.ask.map_or("N/A".to_string(), |v| format!("{}", v))).into(),
            ColumnKind::AskSize => {
                text(row.ask_size.map_or("N/A".to_string(), |v| format!("{}", v))).into()
            }
            ColumnKind::Vol24h => {
                text(row.vol24h.map_or("N/A".to_string(), |v| format!("{}", v))).into()
            }
            ColumnKind::VolumeQuote => {
                text(row.volume_quote.unwrap_or_default().to_string()).into()
            }
            ColumnKind::OpenInterest => {
                text(row.open_interest.unwrap_or_default().to_string()).into()
            }
            ColumnKind::Open24h => {
                text(row.open24h.map_or("N/A".to_string(), |v| format!("{}", v))).into()
            }
            ColumnKind::High24h => {
                text(row.high24h.map_or("N/A".to_string(), |v| format!("{}", v))).into()
            }
            ColumnKind::Low24h => {
                text(row.low24h.map_or("N/A".to_string(), |v| format!("{}", v))).into()
            }
            ColumnKind::LastSize => text(
                row.last_size
                    .map_or("N/A".to_string(), |v| format!("{}", v)),
            )
            .into(),
            ColumnKind::FundingRate => {
                text(row.funding_rate.unwrap_or_default().to_string()).into()
            }
            ColumnKind::FundingRatePrediction => {
                text(row.funding_rate_prediction.unwrap_or_default().to_string()).into()
            }
            ColumnKind::Suspended => text(
                row.suspended
                    .map_or("N/A".to_string(), |v| format!("{}", v)),
            )
            .into(),
            ColumnKind::IndexPrice => text(
                row.index_price
                    .map_or("N/A".to_string(), |v| format!("{}", v)),
            )
            .into(),
            ColumnKind::PostOnly => text(
                row.post_only
                    .map_or("N/A".to_string(), |v| format!("{}", v)),
            )
            .into(),
            ColumnKind::Change24h => text(
                row.change24h
                    .map_or("N/A".to_string(), |v| format!("{}", v)),
            )
            .into(),
        };

        container(content).width(Length::Fill).center_y(32).into()
    }

    fn footer(&'a self, _col_index: usize, _rows: &'a [Self::Row]) -> Option<Element<'a, Message>> {
        Some(horizontal_space().into())
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn resize_offset(&self) -> Option<f32> {
        self.resize_offset
    }
}
