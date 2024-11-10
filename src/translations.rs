// Copyright (C) Pavel Grebnev 2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

use crate::user_settings::*;
use std::collections::HashMap;

pub type Translations = HashMap<String, HashMap<&'static str, &'static str>>;

pub fn translate(
    key: &str,
    translations: &Translations,
    user_settings: &UserSettings,
) -> &'static str {
    translations[&user_settings.language_key][key]
}
