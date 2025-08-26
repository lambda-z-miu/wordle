use std::collections::HashMap;
use egui::Pos2;
use egui::Vec2;
use egui::Color32;

#[derive(PartialEq,Copy,Clone)]
pub enum Color{
    RED,
    YELLOW,
    GREEN,
    GREY,
}

#[derive(Default)]
pub struct MyApp {
    pub board: [[bool; 5]; 6],     // 5列x6行, true = 已点击

    pub board_letter : [[Option<char> ; 5 ] ; 6 ],
    pub board_color  : [[Option<Color> ; 5 ] ; 6 ],

    // pub selected_key: Option<char>, // 最近点击的字母
    // pub difficulty: String,         // 下拉菜单选择

    pub row_full : [bool ; 6],
    pub row_emp  : [bool ; 6],
    pub row_lock : [bool ; 6],

    pub game_state : GameState,
    pub config : MergedConfig,
    pub entered : bool,

    pub confetti: Vec<Confetti>,
    pub time: f32,

    pub winflag : bool,

}

pub struct Confetti {
    pub pos: Pos2,
    pub vel: Vec2,
    pub color: Color32,
    pub lifetime: f32,
}


#[derive(Default)]
pub struct GameState {
    pub word : String,

    pub final_set : Vec<String>,
    pub acc_set : Vec<String>,
    pub days : u32,

    pub trys : Vec<(String,[Color;5])>,
    pub alphabet : HashMap<char,Color>,
    
}

#[derive(Default)]
pub struct MergedConfig {
    pub is_tty: bool,
    pub random: bool,
    pub difficult: bool,
    pub stats : bool,
    pub day : Option<u32>,
    pub seed: Option<u64>,
    pub final_set_path : String,
    pub acc_set_path : String,
    pub state_path: Option<String>,
    pub given_word: Option<String>,
    pub ui : bool,
}