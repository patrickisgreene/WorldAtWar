use serde::{Serialize, Deserialize};
use bevy_color::{prelude::*, palettes::tailwind};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum CombatantColor {
    Amber,
    Blue,
    Cyan,
    Emerald,
    Fuchsia,
    Gray,
    Green,
    Indigo,
    Lime,
    Neutral,
    Orange,
    Pink,
    Purple,
    Red,
    Rose,
    Sky,
    Slate,
    Stone,
    Teal,
    Violet,
    Yellow,
    Zinc,
}

impl CombatantColor {
    pub fn background(&self) -> Srgba {
        use CombatantColor::*;
        match self {
            Amber => tailwind::AMBER_900,
            Blue => tailwind::BLUE_900,
            Cyan => tailwind::CYAN_900,
            Emerald => tailwind::EMERALD_900,
            Fuchsia => tailwind::FUCHSIA_900,
            Gray => tailwind::GRAY_900,
            Green => tailwind::GREEN_900,
            Indigo => tailwind::INDIGO_900,
            Lime => tailwind::LIME_900,
            Neutral => tailwind::NEUTRAL_900,
            Orange => tailwind::ORANGE_900,
            Pink => tailwind::PINK_900,
            Purple => tailwind::PURPLE_900,
            Red => tailwind::RED_900,
            Rose => tailwind::ROSE_900,
            Sky => tailwind::SKY_900,
            Slate => tailwind::SLATE_900,
            Stone => tailwind::STONE_900,
            Teal => tailwind::TEAL_900,
            Violet => tailwind::VIOLET_900,
            Yellow => tailwind::YELLOW_900,
            Zinc => tailwind::ZINC_900,
        }
    }

    pub fn color(&self) -> Srgba {
        use CombatantColor::*;
        match self {
            Amber => tailwind::AMBER_500,
            Blue => tailwind::BLUE_500,
            Cyan => tailwind::CYAN_500,
            Emerald => tailwind::EMERALD_500,
            Fuchsia => tailwind::FUCHSIA_500,
            Gray => tailwind::GRAY_500,
            Green => tailwind::GREEN_500,
            Indigo => tailwind::INDIGO_500,
            Lime => tailwind::LIME_500,
            Neutral => tailwind::NEUTRAL_500,
            Orange => tailwind::ORANGE_500,
            Pink => tailwind::PINK_500,
            Purple => tailwind::PURPLE_500,
            Red => tailwind::RED_500,
            Rose => tailwind::ROSE_500,
            Sky => tailwind::SKY_500,
            Slate => tailwind::SLATE_500,
            Stone => tailwind::STONE_500,
            Teal => tailwind::TEAL_500,
            Violet => tailwind::VIOLET_500,
            Yellow => tailwind::YELLOW_500,
            Zinc => tailwind::ZINC_500,
        }
    }

    pub fn highlight(&self) -> Srgba {
        use CombatantColor::*;
        match self {
            Amber => tailwind::AMBER_300,
            Blue => tailwind::BLUE_300,
            Cyan => tailwind::CYAN_300,
            Emerald => tailwind::EMERALD_300,
            Fuchsia => tailwind::FUCHSIA_300,
            Gray => tailwind::GRAY_300,
            Green => tailwind::GREEN_300,
            Indigo => tailwind::INDIGO_300,
            Lime => tailwind::LIME_300,
            Neutral => tailwind::NEUTRAL_300,
            Orange => tailwind::ORANGE_300,
            Pink => tailwind::PINK_300,
            Purple => tailwind::PURPLE_300,
            Red => tailwind::RED_300,
            Rose => tailwind::ROSE_300,
            Sky => tailwind::SKY_300,
            Slate => tailwind::SLATE_300,
            Stone => tailwind::STONE_300,
            Teal => tailwind::TEAL_300,
            Violet => tailwind::VIOLET_300,
            Yellow => tailwind::YELLOW_300,
            Zinc => tailwind::ZINC_300,
        }
    }
}