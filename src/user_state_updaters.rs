// Copyright (C) Pavel Grebnev 2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

use crate::json_file_updater::{JsonFileUpdater, UpdateResult};
use serde_json::Value as JsonValue;

static VERSION_FIELD_NAME: &str = "version";
pub static LATEST_SAVE_VERSION: &str = "0.2.0";

pub fn update_user_states_to_the_latest_version(user_states_json: &mut JsonValue) -> UpdateResult {
    let version = user_states_json[VERSION_FIELD_NAME].as_str();
    if let Some(version) = version {
        if version == LATEST_SAVE_VERSION {
            return UpdateResult::NoUpdateNeeded;
        }
    }

    let json_file_updater = register_json_updaters();

    json_file_updater.update_json(user_states_json)
}

fn register_json_updaters() -> JsonFileUpdater {
    let mut json_file_updater = JsonFileUpdater::new(VERSION_FIELD_NAME);

    json_file_updater.add_update_function("0.1.0", |_|{});
    json_file_updater.add_update_function("0.2.0", v0_2_0_rename_4_30_mangan_to_kiriage_mangan);
    // add update functions above this line
    // don't forget to update LATEST_SAVE_VERSION at the beginning of the file

    json_file_updater
}

fn v0_2_0_rename_4_30_mangan_to_kiriage_mangan(user_states_json: &mut JsonValue) {
    for user_state in user_states_json["states"].as_object_mut().unwrap().values_mut() {
        let user_state = user_state.as_object_mut().unwrap();
        let scoring_settings = user_state.get_mut("scoring_settings").unwrap().as_object_mut().unwrap();
        let use_4_30_mangan = scoring_settings.remove("use_4_30_mangan").unwrap().as_bool().unwrap();
        scoring_settings.insert("use_kiriage_mangan".to_string(), JsonValue::Bool(use_4_30_mangan));
    }
}
