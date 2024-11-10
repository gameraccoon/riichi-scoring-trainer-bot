// Copyright (C) Pavel Grebnev 2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

use crate::hand_score::ScoringSettings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub scoring_settings: ScoringSettings,
    pub language_key: String,
}

pub fn get_default_settings() -> UserSettings {
    UserSettings {
        scoring_settings: ScoringSettings {
            use_kiriage_mangan: false,
            use_honba: false,
            use_kazoe_yakuman: true,
        },
        language_key: "en".to_string(),
    }
}
