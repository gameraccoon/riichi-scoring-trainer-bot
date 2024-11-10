// Copyright (C) Pavel Grebnev 2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

mod hand_score;
mod json_file_updater;
mod telegram_bot;
mod translations;
mod user_settings;
mod user_state;
mod user_state_updaters;

extern crate rand;

#[tokio::main]
async fn main() {
    telegram_bot::run_telegram_bot().await;
}
