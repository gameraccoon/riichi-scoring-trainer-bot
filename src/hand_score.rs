// Copyright (C) Pavel Grebnev 2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScoringSettings {
    pub use_4_30_mangan: bool,
    pub use_honba: bool,
    pub use_kazoe_yakuman: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct HandScore {
    pub total_dealer: u16,
    pub total_other: u16,
    pub han: u8,
    pub fu: u8,
    pub honba: u8,
    pub ron: bool,
    pub is_dealer: bool,
}

impl HandScore {
    fn generate_gaussian(mean: f64, std_dev: f64, min: f64, max: f64) -> f64 {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut s = 0.0;
        while s >= 1.0 || s == 0.0 {
            x = 2.0 * rand::random::<f64>() - 1.0;
            y = 2.0 * rand::random::<f64>() - 1.0;
            s = x * x + y * y;
        }
        let result = mean + std_dev * x * (-2.0 * s.ln() / s).sqrt();

        if result < min {
            min
        } else if result > max {
            max
        } else {
            result
        }
    }

    pub fn generate_winning_hand(settings: ScoringSettings) -> HandScore {
        let is_ron = rand::random::<bool>();
        let is_chiitoi = rand::random::<f32>() < 0.0252;

        let min_han = if is_chiitoi { 2.0 } else { 1.0 };

        // bell curve for han with mean 3 and standard deviation 2
        let han = HandScore::generate_gaussian(3.0, 2.0, min_han, 20.0) as u8;

        let (min_fu, max_fu) = if is_chiitoi {
            (25.0, 25.0)
        } else {
            let min = if han == 1 { 30.0 } else { 20.0 };
            let max = if is_ron { 110.0 } else { 100.0 };
            (min, max)
        };

        // bell curve for fu with mean 30 and standard deviation 10
        let fu_not_rounded = Self::generate_gaussian(30.0, 10.0, min_fu, max_fu) as u8;
        let fu = if min_fu == 25.0 {
            25
        } else {
            // round up to the nearest 10
            (fu_not_rounded + 5) / 10 * 10
        };

        // bell curve for honba with mean 0 and standard deviation 2
        let honba = if settings.use_honba {
            Self::generate_gaussian(0.0, 2.0, 0.0, 12.0) as u8
        } else {
            0
        };

        let mut score = HandScore {
            total_dealer: 0,
            total_other: 0,
            han,
            fu,
            honba,
            ron: if rand::random::<bool>() { true } else { false },
            is_dealer: if rand::random::<i8>() % 4 == 0 {
                true
            } else {
                false
            },
        };

        score.calc_totals(settings);

        score
    }

    pub fn calc_totals(&mut self, settings: ScoringSettings) {
        // 0 is 5 han, max is 11 han
        const LIMITS: [u16; 7] = [2000, 3000, 3000, 4000, 4000, 4000, 6000];

        // 0 is 5 han, max is 13 han
        const LIMITS_KAZOE: [u16; 9] = [2000, 3000, 3000, 4000, 4000, 4000, 6000, 6000, 8000];

        let mut base: u16 = 0;
        if self.han > 5 {
            // limit hands
            let index = (self.han - 5) as usize;

            if settings.use_kazoe_yakuman {
                if index >= LIMITS_KAZOE.len() {
                    base = LIMITS_KAZOE[LIMITS_KAZOE.len() - 1];
                } else {
                    base = LIMITS_KAZOE[index];
                }
            } else {
                if index >= LIMITS.len() {
                    base = LIMITS[LIMITS.len() - 1];
                } else {
                    base = LIMITS[index];
                }
            }
        } else if settings.use_4_30_mangan && (self.han == 4 && self.fu == 30 || self.han == 3 && self.fu == 60) {
            // 4 han 30 fu or 3 han 60 fu is mangan when kiriage mangan is on
            base = 2000;
        } else {
            // normal hands
            base = std::cmp::min(self.fu as u16 * 2u16.pow(2 + self.han as u32), 2000);
        }

        if self.ron {
            if self.is_dealer {
                self.total_dealer = 0;
                self.total_other = base * 6;
            } else {
                self.total_dealer = 0;
                self.total_other = base * 4;
            }
        } else {
            if self.is_dealer {
                self.total_dealer = 0;
                self.total_other = base * 2;
            } else {
                self.total_dealer = base * 2;
                self.total_other = base;
            }
        }

        // round totals up to the next 100
        self.total_dealer = (self.total_dealer + 99) / 100 * 100;
        self.total_other = (self.total_other + 99) / 100 * 100;

        // add honba
        self.total_dealer += if self.total_dealer != 0 { self.honba as u16 * 100 } else { 0 };
        self.total_other += self.honba as u16 * if self.ron { 300 } else { 100 };
    }
}
