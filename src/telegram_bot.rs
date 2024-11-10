// Copyright (C) Pavel Grebnev 2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

use crate::hand_score::HandScoreData;
use crate::translations::*;
use crate::user_settings::*;
use crate::user_state::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use teloxide::prelude::*;

static USER_STATES_PATH: &str = "./data/user_states.json";

fn read_telegram_token() -> String {
    fs::read_to_string("./telegramApiToken.txt")
        .expect("Can't read file \"telegramApiToken.txt\", please make sure the file exists and contains the bot API Token")
}

struct StaticData {
    translations: Translations,
}

#[derive(Clone)]
struct Response {
    text: String,
    image: Option<teloxide::types::InputFile>,
}

fn text_response(text: &str) -> Vec<Response> {
    [Response {
        text: text.to_string(),
        image: None,
    }]
    .to_vec()
}

fn generate_new_hand_text(
    hand_score: &mut Option<HandScoreData>,
    settings: &UserSettings,
) -> String {
    *hand_score = Some(HandScoreData::generate_winning_hand(
        settings.scoring_settings,
    ));
    let score = hand_score.as_ref().unwrap();
    format!(
        "{} han{}\n{}\n{}{}",
        score.han,
        if score.han >= 5 {
            "".to_string()
        } else {
            format!("\n{} fu", score.fu)
        },
        if score.is_dealer {
            "dealer"
        } else {
            "non-dealer"
        },
        if score.ron { "ron" } else { "tsumo" },
        if score.honba > 0 {
            format!("\n{} honba", score.honba)
        } else {
            "".to_string()
        }
    )
}

fn text_response_str(text: String) -> Vec<Response> {
    [Response { text, image: None }].to_vec()
}

fn single_text_response_str(text: String) -> Response {
    Response { text, image: None }
}

fn single_text_response(text: &str) -> Response {
    Response {
        text: text.to_string(),
        image: None,
    }
}

fn process_user_message(
    user_state: &mut UserState,
    message: &Message,
    static_data: &StaticData,
) -> Vec<Response> {
    let Some(message_text) = &message.text() else {
        return text_response("No message received");
    };

    const NO_GAME_IN_PROGRESS_MESSAGE: &str =
        "No game is in progress, send /start to start a new game";
    let settings = &mut user_state.settings;
    let opt_hand_score = &mut user_state.hand_score;
    let mut message_split = message_text.split_whitespace();

    match message_split.next() {
        Some("/start") => {
            return text_response_str(generate_new_hand_text(opt_hand_score, settings))
        }
        Some("/settings") => {
            return text_response(&format!(
                "
/toggle_kiriage_mangan - turn {} counting 4 han 30 fu and 3 han 60 fu as mangan
/toggle_honba - turn {} honba counting
/toggle_kazoe - turn {} counting kazoe yakuman",
                if user_state.settings.scoring_settings.use_4_30_mangan {
                    "off"
                } else {
                    "on"
                },
                if user_state.settings.scoring_settings.use_honba {
                    "off"
                } else {
                    "on"
                },
                if user_state.settings.scoring_settings.use_kazoe_yakuman {
                    "off"
                } else {
                    "on"
                }
            ))
        }
        Some("/toggle_kiriage_mangan") => {
            settings.scoring_settings.use_4_30_mangan = !settings.scoring_settings.use_4_30_mangan;
            user_state.settings_unsaved = true;
            return text_response_str(format!(
                "4 han 30 fu is now {}counted as mangan",
                if settings.scoring_settings.use_4_30_mangan {
                    ""
                } else {
                    "not "
                }
            ));
        }
        Some("/toggle_honba") => {
            settings.scoring_settings.use_honba = !settings.scoring_settings.use_honba;
            user_state.settings_unsaved = true;
            return text_response_str(format!(
                "Honba is now {}used",
                if settings.scoring_settings.use_honba {
                    ""
                } else {
                    "not "
                }
            ));
        }
        Some("/toggle_kazoe") => {
            settings.scoring_settings.use_kazoe_yakuman =
                !settings.scoring_settings.use_kazoe_yakuman;
            user_state.settings_unsaved = true;
            return text_response_str(format!(
                "Kazoe yakuman is now {}counted",
                if settings.scoring_settings.use_kazoe_yakuman {
                    ""
                } else {
                    "not "
                }
            ));
        }
        Some("/help") => {
            return text_response("This bot helps training score counting in riichi mahjong.\n\nSend /start to start a new game, then send the score in the format 1000 or 1000/2000 to check if it's correct.\n\nSend /settings to see and change the settings");
        }
        Some(_) => {}
        None => {}
    }

    let Some(hand_score) = opt_hand_score else {
        return text_response(NO_GAME_IN_PROGRESS_MESSAGE);
    };

    let mut score_parts = message_text.split(|c| c == '/' || c == ' ');

    let first_part = score_parts.next();
    let second_part = score_parts.next();

    if let Some(others_score) = first_part {
        let total_others = if let Ok(others_score) = others_score.parse::<u16>() {
            others_score
        } else {
            return text_response("Failed to parse score, format is 1000 or 1000/2000");
        };

        let total_dealer = if let Some(total_dealer) = second_part {
            if let Ok(total_dealer) = total_dealer.parse::<u16>() {
                total_dealer
            } else {
                return text_response("Failed to parse score, format is 1000 or 1000/2000");
            }
        } else {
            0
        };

        let totals = hand_score.calculate_totals(settings.scoring_settings);

        if total_others == totals.others && total_dealer == totals.dealer
            || total_others == totals.dealer && total_dealer == totals.others
        {
            text_response_str(
                "Correct score\n\nNext hand:\n".to_string()
                    + &generate_new_hand_text(opt_hand_score, settings),
            )
        } else {
            if totals.dealer == 0 {
                text_response_str(
                    format!(
                        "Incorrect score, correct score is\n{}\n\nNext hand:\n",
                        totals.others
                    ) + &generate_new_hand_text(opt_hand_score, settings),
                )
            } else {
                text_response_str(
                    format!(
                        "Incorrect score, correct score is\n{}/{}\n\nNext hand:\n",
                        totals.others, totals.dealer
                    ) + &generate_new_hand_text(opt_hand_score, settings),
                )
            }
        }
    } else {
        text_response("Failed to parse score, format is 1000 or 1000/2000")
    }
}

fn load_translations() -> Translations {
    let mut translations = HashMap::new();

    {
        translations.insert(
            "ene".to_string(),
            HashMap::from([("tenpai_hand", "The hand is ready now")]),
        );
    }

    {
        translations.insert(
            "enj".to_string(),
            HashMap::from([("tenpai_hand", "Tenpai")]),
        );
    }

    translations
}

pub async fn run_telegram_bot() {
    pretty_env_logger::init();
    log::info!("Starting the bot");

    let token = read_telegram_token();

    let bot = Bot::new(token);

    type SharedUserStates = Arc<UserStates>;
    type SharedStaticData = Arc<StaticData>;

    let user_states =
        SharedUserStates::new(read_user_states_from_file(Path::new(USER_STATES_PATH)));
    let static_data = SharedStaticData::new(StaticData {
        translations: load_translations(),
    });

    let handler = Update::filter_message().endpoint(
        |bot: Bot,
         user_states: SharedUserStates,
         static_data: SharedStaticData,
         message: Message| async move {
            let user_state: &mut UserState = &mut user_states
                .states
                .entry(message.chat.id)
                .or_insert_with(|| get_default_user_state());

            let responses = process_user_message(user_state, &message, &static_data);
            if user_state.settings_unsaved {
                save_single_user_state(Path::new(USER_STATES_PATH), message.chat.id, &user_state);
                user_state.settings_unsaved = false;
            }
            for response in responses {
                let send_result = if let Some(image) = response.image {
                    let text = response.text;
                    let mut send_photo = bot.send_photo(message.chat.id, image);
                    if !text.is_empty() {
                        send_photo.caption = Some(text);
                    }
                    send_photo.send().await
                } else {
                    bot.send_message(message.chat.id, response.text).await
                };

                if send_result.is_err() {
                    log::error!("Failed to send photo: {:?}", send_result.err());
                }
            }
            respond(())
        },
    );

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![user_states.clone(), static_data.clone()])
        .build()
        .dispatch()
        .await;
}
