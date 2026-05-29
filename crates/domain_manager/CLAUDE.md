# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Domain Manager is a Rust-based GUI application for managing domains and DNS records across multiple providers (Aliyun DNS, Cloudflare). Built with the Iced GUI framework.

## Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run application
cargo run

# Run tests
cargo test --verbose

# Run single test
cargo test test_name -- --nocapture

# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Quick check (fmt + clippy)
make check
```

## Architecture

### Entry Point
- `src/main.rs` - Application entry point using Iced framework with async main

### Core Modules
- **`src/gui/`** - GUI layer using Iced
  - `manager_v2.rs` - Main application state and message handling (V2 architecture)
  - `manager.rs` - Legacy main manager (being refactored)
  - `state/` - Application state management (`AppState`, `UiState`, `DataState`)
  - `handlers/` - Event handlers for different message types
  - `services/` - Business logic services (Domain, DNS, Sync, Config, Database)
  - `components/` - Reusable UI components
  - `pages/` - Page-level views
  - `styles/` - Theme system with multiple built-in themes

### API Layer
- **`src/api/`** - DNS provider API integrations
  - `provider/aliyun.rs` - Aliyun DNS API
  - `provider/cloudflare_provider.rs` - Cloudflare API
  - `dns_client.rs` - Unified DNS client interface

### Storage Layer
- **`src/storage/`** - Data persistence
  - `entities/` - SeaORM entity definitions (Domain, DnsRecord, Account, Provider)
  - `migration/` - Database migrations using SeaORM
  - `database.rs` - SQLite/PostgreSQL connection management

### Configuration
- `src/configs/` - Application configuration
  - `gui_config.rs` - GUI settings (window state, theme, language)
  - `database.rs` - Database connection config

## Key Patterns

### Message Handling
Messages are categorized via `MessageCategory` enum in `handlers/message_handler.rs`:
- `App`, `Database`, `Sync`, `Domain`, `Dns`, `Window`, `Ui`, `Provider` variants

### State Management
- `AppState` in `state/app_state.rs` holds all application state
- Updates go through `StateUpdate` enum (`UiUpdate`, `DataUpdate`)

### Service Layer
Services implement traits from `services/mod.rs`:
- `DomainServiceTrait`, `DnsServiceTrait`, `SyncServiceTrait`, `DatabaseServiceTrait`, `ConfigServiceTrait`
- Managed by `ServiceManager`

### Refactoring Status
- V2 architecture (`manager_v2.rs`) is the active development target
- Legacy `manager.rs` (2398 lines) is being incrementally refactored per `REFACTOR_PLAN.md`

## Internationalization

- Uses `rust-i18n` with locale files in `locales/` directory
- Default locale: English (`en`), also supports Chinese (`zh_CN`)
- Translation function: `get_text("key")`

## Database

- Default: SQLite (stored in app data directory)
- Optional: PostgreSQL via config in `application.yaml`
- Migrations in `src/storage/migration/` using SeaORM
