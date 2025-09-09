use std::collections::HashSet;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Styled {
    content: String,
    color: Color,
    effects: HashSet<Effect>,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    #[default]
    White = 7,
}

use std::mem::transmute;

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => unsafe { transmute::<u8, Self>(value) },
            _ => Self::default(),
        }
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        match &value.to_lowercase()[..] {
            "red" | "r" | "1" => Self::Red,
            "green" | "g" | "grn" | "2" => Self::Green,
            "yellow" | "y" | "ylw" | "ylo" | "3" => Self::Yellow,
            "blue" | "blu" | "ble" | "4" => Self::Blue,
            "mgnt" | "magenta" | "mgt" | "mgn" | "mag" | "magen" | "magnt" | "5" => Self::Magenta,
            "cyan" | "c" | "cyn" | "can" | "yan" | "6" => Self::Cyan,
            "white" | "w" | "wht" | "whit" | "wte" | "7" => Self::White,
            "black" | "blk" | "blac" | "blck" | "0" | "bl" => Self::Black,
            _ => Self::default(),
        }
    }
}

impl From<&Color> for &str {
    fn from(value: &Color) -> &'static str {
        match value {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Effect {
    #[default]
    Clear = 0,
    Bold = 1,
    Italic = 3,
    Underline = 4,
    StrikeThrough = 9,
}

impl From<u8> for Effect {
    fn from(value: u8) -> Self {
        match value {
            0 | 1 | 3 | 4 | 9 => unsafe { transmute::<u8, Self>(value) },
            _ => Self::default(),
        }
    }
}

impl From<&str> for Effect {
    fn from(value: &str) -> Self {
        match &value.to_lowercase()[..] {
            "bold" | "b" | "bld" | "bol" | "1" => Self::Bold,
            "clr" | "c" | "clear" | "0" => Self::Clear,
            "strikethrough" | "st" | "strike_through" | "strk_throu" | "stkthr" | "9" => {
                Self::StrikeThrough
            }
            "underline" | "udrln" | "ul" | "under" | "undln" | "undline" | "4" => Self::Underline,
            "italic" | "itlc" | "i" | "it" | "itl" | "3" => Self::Italic,
            _ => Self::default(),
        }
    }
}

impl From<&Effect> for &str {
    fn from(value: &Effect) -> &'static str {
        match value {
            Effect::Bold => "1",
            Effect::Clear => "0",
            Effect::StrikeThrough => "9",
            Effect::Italic => "3",
            Effect::Underline => "4",
        }
    }
}

impl Styled {
    pub fn color(mut self, clr: impl Into<Color>) -> Self {
        self.color = clr.into();

        self
    }

    fn color_as_str(&self) -> &str {
        (&self.color).into()
    }

    fn effects_to_string(&self) -> String {
        self.effects
            .iter()
            .map(|e| e.into())
            .fold(String::new(), |acc, e| acc + ";" + e)
    }

    pub fn effect(mut self, efct: impl Into<Effect>) -> Self {
        self.effects.insert(efct.into());

        self
    }

    pub fn effects<T, I>(mut self, efcts: T) -> Self
    where
        T: IntoIterator<Item = I>,
        I: Into<Effect>,
    {
        efcts.into_iter().for_each(|e| {
            self.effects.insert(e.into());
        });

        self
    }

    pub fn remove(mut self, efct: impl Into<Effect>) -> Self {
        self.effects.remove(&efct.into());

        self
    }

    pub fn fmt(&self, content: impl AsRef<str>) -> String {
        format!(
            "\x1b[{}{}m{}\x1b[0m",
            self.color_as_str(),
            self.effects_to_string(),
            content.as_ref()
        )
    }
}
