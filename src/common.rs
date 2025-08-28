use egui::Color32;
use egui::Pos2;
use egui::Vec2;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone)]
pub enum Color {
    RED,
    YELLOW,
    GREEN,
    GREY,
}

#[derive(Default)]
pub struct MyApp {
    pub board_letter: [[Option<char>; 5]; 6],
    pub board_color: [[Option<Color>; 5]; 6],

    pub row_lock: [bool; 6],

    pub game_state: GameState,
    pub config: MergedConfig,
    pub entered: bool,

    pub confetti: Vec<Confetti>,
    pub time: f32,

    pub winflag: bool,
}

pub struct Confetti {
    pub pos: Pos2,
    pub vel: Vec2,
    pub color: Color32,
    pub lifetime: f32,
}

#[derive(Default)]
pub struct GameState {
    pub word: String,

    pub final_set: Vec<String>,
    pub acc_set: Vec<String>,
    pub days: u32,

    pub trys: Vec<(String, [Color; 5])>,
    pub alphabet: HashMap<char, Color>,
}

#[derive(Default)]
pub struct MergedConfig {
    pub random: bool,
    pub difficult: bool,
    pub stats: bool,
    pub day: Option<u32>,
    pub seed: Option<u64>,
    pub final_set_path: String,
    pub acc_set_path: String,
    pub state_path: Option<String>,
    pub given_word: Option<String>,
    pub ui: bool,
}
