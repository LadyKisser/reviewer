# 🌟 Reviewer - Discord Review System

<div align="center">

<img src="./assets/logo.jpg" alt="ReviewBot Logo" style="border-radius: 50%; width: 150px; height: 150px; object-fit: cover;" />

<p align="center">
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
<a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust"></a>
<a href="https://discord.com"><img src="https://img.shields.io/badge/discord-bot-7289da.svg" alt="Discord"></a>
<a href="https://github.com/LadyKisser/reviewer"><img src="https://img.shields.io/badge/status-work%20in%20progress-yellow.svg" alt="Status"></a>
</p>

*A Discord bot for managing user and server reviews, written in Rust* 🦀

</div>

## ⚠️ Work in Progress

> **Note:** This bot is currently under development and is **not** production-ready. As my second/third attempt at writing a Discord bot in Rust, there might be bugs, inefficiencies, or unfinished features.

## ✨ Features

### Current Features
- 📝 User Review System
  - Rate users from 1-5 stars
  - Add optional comments
  - Review history pagination

- 🏠 Server Review System
  - Rate Discord servers
  - Server-specific review tracking
  - Paginated review history

- 🔄 Cache System
  - Redis integration
  - Efficient rating caching
  - Reduced database load

### Planned Features
- [ ] Review moderation system
- [ ] Review analytics and statistics
- [ ] Custom review categories
- [ ] Review reactions
- [ ] API integration improvements
- [ ] Rate limiting
- [ ] User blacklisting
- [ ] Image attachments support
  - Multiple images per review
  - Automatic WebP conversion
  - URL validation
  - Image cleanup system

## 🛠️ Technology Stack

- **Language:** Rust 🦀
- **Framework:** [Poise](https://github.com/serenity-rs/poise)
- **Database:** PostgreSQL
- **Cache:** Redis
- **API:** Axum

## 📋 Prerequisites

- Rust 1.75 or higher
- PostgreSQL
- Redis
- Discord Bot Token

## 🚀 Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/LadyKisser/reviewer.git
   cd reviewer
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Set up the database**
   ```bash
   psql -U your_username -d your_database -a -f schema.sql
   ```

4. **Build and run**
   ```bash
   cargo build --release
   cargo run --release
   ```

## 🎨 Commands

| Command | Description |
|---------|-------------|
| `/review user @user` | Review a user |
| `/review server https://discord.gg/example` | Review a server |

## 🚧 Known Issues

- Some edge cases in review pagination aren't handled
- Server review system needs more testing
- Cache system might need optimization
- API endpoints need more documentation

## 📝 Contributing

As this is a learning project, contributions are welcome but please note that the code might not be optimal. Feel free to:
- Report bugs
- Suggest features
- Submit pull requests
- Share improvements

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

Made with ❤️ and lots of 🍷

*This is a learning project and might contain bugs or unfinished features*

</div>