# 🦑 Krader - Experimental Iced Trading App

*Woof! This is a playground to learn Rust, Iced, and real-time trading UIs.*

**Krader** is a small, experimental desktop application built to showcase how to use **Rust** and **Iced** (v0.13.1) to interact with the Kraken API. It’s not production software—think of it as a learning project or proof-of-concept for crafting real-time data-driven desktop UIs.

---

## 🚀 What's inside?

- **Iced Basics**: MVU architecture (Model, Message, update, view) and how to wire up an Iced app.
- **Async in Rust**: Using Tokio and WebSocket streams for live market data.
- **Custom Widgets**: Building heatmaps, sparklines, and draggable markers in Iced’s canvas.
- **Local Caching**: Simple in-memory or disk-backed storage to avoid refetching every tick.
- **Cross-Platform Packaging**: How to bundle your app on macOS, Windows, and Linux.

---

## 📦 Features (Proof-of-Concept)

- **Live Ticker**: Stream real-time price updates for a watchlist of crypto pairs.
- **Order Book**: Visual depth map and numeric bids/asks table.
- **Chart Trading**: Place and modify mock orders directly on a candlestick chart.
- **Kraken API**: Fetch public market data via WebSocket or REST endpoints.
- **Dark Theme**: A simple dark-mode UI with neon-green/red accents.

---

## 🏁 Getting Started

### Prerequisites
- **Rust** (1.70+)
- **cargo** + **cargo-make** (optional)
- **KRAKEN_API_KEY**: Your API key, you need to edit the .env file and add it there to fetch real data.

### Installation & Run

```bash
git clone https://github.com/wassimans/krader.git
cd krader
a # if using cargo-make
cargo make setup # installs dependencies
cargo run --release
```


---

## 🛠 Project Structure

```
krader/
├─ src/                # Main Rust + Iced source
├─ assets/             # Icons & UI assets
├─ config/             # (Optional) config files for API keys or endpoints
├─ docs/               # Guides, notes, and roadmap sketches
└─ Makefile or ./xtask # Task automation (build, package, lint)
```

---

## 📅 Experimental Roadmap

1. **Phase 1**: Live Ticker panel: Building
2. **Phase 2**: Order Book viewer: Todo
3. **Phase 3**: Chart Trading with draggable markers: Todo
4. **Phase 4**: Stitch panels into one window + theming: Todo
5. **Phase 5**: Package builds for macOS/Win/Linux: Todo

---

## 🤝 Contributing & Learning

- This repo is for learning purposes.
- Copy code, break it, fix it, learn by doing.


