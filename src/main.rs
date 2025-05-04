use std::time::Duration;

use iced::{application, time::every, widget::Text, Element, Subscription, Task, Theme};

fn main() -> iced::Result {
    application(Krader::title, Krader::update, Krader::view)
        .subscription(Krader::subscription)
        .theme(Krader::theme)
        .run_with(Krader::new)
}

pub struct Krader {
    seconds: u64,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl Krader {
    fn new() -> (Self, Task<Message>) {
        (Krader { seconds: 0 }, Task::none())
    }

    fn title(&self) -> String {
        "🦑 Krader".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.seconds += 1;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        Text::new(format!("Hello, Krader! {} s", self.seconds))
            .size(40)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        every(Duration::from_secs(1)).map(|_| Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
