# 🦑 Krader - Experimental Iced Kraken Trading Dashboard

*Woof! This is a playground to learn Rust, Iced, and real-time trading UIs.*

**Krader** is a small, experimental desktop application built to showcase how to use **Rust** and **Iced** (v0.13.1) to interact with the Kraken API. It’s not production software—think of it as a learning project or proof-of-concept for crafting real-time data-driven desktop UIs.

---

## 🚀 What's inside?

- **Iced Basics**: MVU architecture (Model, Message, update, view) and how to wire up an Iced app.
- **Async in Rust**: Using Tokio and WebSocket streams for live market data.

---

## 📦 Features (Proof-of-Concept)

- **Live Ticker**: Stream real-time price updates for a watchlist of crypto pairs.
- **Kraken API**: Fetch public market data via REST endpoints.
- **Dark Theme**: A simple dark-mode UI with neon-green/red accents.

---

## 🏁 Getting Started

### Prerequisites
- **Rust** (1.70+)
- **cargo**

### Installation & Run

```bash
git clone https://github.com/wassimans/krader.git
cd krader
cargo run --release
```

---

## 📅 Experimental Roadmap

1. **Phase 1**: Live Ticker panel: ✅  Done
2. **Phase 2**: Watchlist: ✅  Done

---

