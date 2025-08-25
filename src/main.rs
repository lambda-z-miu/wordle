mod parseconfig;
use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::collections::HashSet;
use rand::seq::SliceRandom; 
use rand::SeedableRng; 
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use serde_json;

use std::io;
use colored::*;
use std::collections::HashMap;
use std::cmp::min;

use eframe::egui;
use egui::StrokeKind::Inside;


#[derive(Default)]
struct MyApp {
    board: [[bool; 5]; 6],     // 5列x6行, true = 已点击
    board_letter : [[Option<char> ; 5 ] ; 6 ],
    selected_key: Option<char>, // 最近点击的字母
    difficulty: String,         // 下拉菜单选择
}

impl MyApp {
    fn add_char(&mut self, new_letter : char ){
        for rowindex in 0..6{
            for colindex in 0..5{
                if self.board_letter[rowindex][colindex]==None{
                    self.board_letter[rowindex][colindex] = Some(new_letter);
                    return;
                }
            }
        }
    }
}




impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut fonts = egui::FontDefinitions::default();
        // 加载外部字体
        fonts.font_data.insert(
            "ARIAL_BOLD".to_owned(),
            egui::FontData::from_static(include_bytes!("../font.TTF")).into(),
        );

        // 在 Proportional 字体族里插入
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "ARIAL_BOLD".to_owned());

        ctx.set_fonts(fonts);


        for i in self.board_letter{
            for j in i{
                if j != None{
                    // println!("{}",j.expect("UR"));
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {

            /* 
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Wordle").size(48.0) .strong());
                ui.add_space(10.0);
            }); */

            ui.input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    if let Some(ch) = text.chars().next() {
                        if ch.is_ascii_alphabetic() {
                                self.add_char(ch.to_ascii_uppercase());
                            }
                        }
                    }
                }
            });
            

            // === 棋盘 (6 行 5 列) ===
            ui.separator();
            ui.add_space(30.0);
            
            // 一些参数
            const COLS: usize = 5;
            const TILE: f32 = 60.0;   // 方块边长
            const GAP: f32 = 5.0;     // 列间距
            const ROW_GAP: f32 = 5.0; // 行间距


            for row in 0..6 {
                // 计算这一行的总宽度（所有方块 + 列间距）
                let total_row_w = COLS as f32 * TILE + (COLS.saturating_sub(1)) as f32 * GAP;
                // 当前可用宽度
                let avail = ui.available_width();
                // 让这一行整体居中所需的左侧留白
                let left_pad = ((avail - total_row_w) * 0.5).max(0.0);

                // 这一行用水平布局：先塞左侧留白，再依次画 5 个方块
                ui.horizontal(|ui| {
                    // 给这一行的左侧留白，从而让整行居中
                        ui.add_space(left_pad);

                    for col in 0..COLS {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(TILE, TILE),
                            egui::Sense::click(),
                        );
                        let painter = ui.painter_at(rect);

                        // 背景色: 已点击绿色，否则灰色
                        if !self.board[row][col]{
                            let fill_color = egui::Color32::WHITE;        // 内部填充白色
                            let stroke_color = egui::Color32::GRAY;       // 边框灰色
                            let stroke = egui::Stroke::new(2.0, stroke_color); // 边框粗细和颜色

                            // 先画填充
                            painter.rect_filled(rect, 5.0, fill_color);
                            painter.rect_stroke(rect, 5.0, stroke,Inside);
                        }
                        else{
                            let fill_color = egui::Color32::GREEN;        // 内部填充白色
                            let stroke_color = egui::Color32::GRAY;       // 边框灰色
                            let stroke = egui::Stroke::new(2.0, stroke_color); // 边框粗细和颜色
                            painter.rect_filled(rect, 5.0, fill_color);
                            painter.rect_stroke(rect, 5.0, stroke,Inside);
                        }

                        
                        let ch = self.board_letter[row][col];
                        if let Some(letter) = ch{
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                letter.to_string(),
                                egui::FontId::proportional(35.0), // 字号
                                egui::Color32::WHITE,             // 白色字
                            );
                        }
                        
                        // 点击切换状态
                        if response.clicked() {
                            self.board[row][col] = !self.board[row][col];
                        }

                        // 列间距（最后一列不用加）
                        if col + 1 != COLS {
                            ui.add_space(GAP);
                        }
                    }
                });

                // 行间距（最后一行可选不加）
                if row + 1 != 6 {
                    ui.add_space(ROW_GAP);
                }
            }

            ui.add_space(30.0);

            ui.separator();

            // === 虚拟键盘 A-Z ===
            ui.label("虚拟键盘:");
            for chunk in ('A'..='Z').collect::<Vec<_>>().chunks(9) {
                ui.horizontal(|ui| {
                    for &ch in chunk {
                        if ui.button(ch.to_string()).clicked() {
                            self.selected_key = Some(ch);
                        }
                    }
                });
            }

            if let Some(ch) = self.selected_key {
                ui.label(format!("最近点击的键: {}", ch));
            }

            ui.separator();

            // === 下拉菜单（难度选择） ===
            egui::ComboBox::from_label("选择难度")
                .selected_text(&self.difficulty)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.difficulty, "简单".to_string(), "简单");
                    ui.selectable_value(&mut self.difficulty, "普通".to_string(), "普通");
                    ui.selectable_value(&mut self.difficulty, "困难".to_string(), "困难");
                });
        });
    }
}







fn main() -> () {



    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Wordle UI Demo",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    ).expect("a");

/* 

    let final_config = parseconfig::parse_config();
    
    let JSON_path = &final_config.state_path;
    let mut state_from_JSON : JSONState =  JSONState {
        total_rounds: None,
        games: Vec::new(),
    }; // set a default empty state

    let mut opt_path = "tmp.json";
    if let Some(JSON_path) = JSON_path{
        if let Some(state) = JSONState::get_state_form_JSON(&JSON_path){
            state_from_JSON = state;
            opt_path = "state_mem.json";
        }
    }
    let mut game_state = generate_game_state(&final_config);

    if final_config.random{
        loop{
            muilti_round(&final_config,&mut game_state);
            state_from_JSON.update_JSONstate(&game_state);
            state_from_JSON.write_to_JSON(opt_path);
            if final_config.stats{
                let stat_data = state_from_JSON.stat();
                stat_data.print();
            }
            println!("insert q to end");
            let mut input = String ::new();
            io::stdin().read_line(&mut input).expect("unable to read");

            if opt_path == "tmp.json"{
                fs::remove_file("tmp.json").expect("Unable to delete tmp");
            }
            

            
            let trimmed_input = input.trim_end();
            if trimmed_input == 'q'.to_string(){
                break;
            }
            reset_game_state(&final_config,&mut game_state);
        }
    }
    else{
        let mut game_state = generate_game_state(&final_config);
        muilti_round(&final_config,&mut game_state);
        state_from_JSON.update_JSONstate(&game_state);
        state_from_JSON.write_to_JSON("state_write.json");
        if final_config.stats{
            let stat_data = state_from_JSON.stat();
            stat_data.print();
        }
    }
*/
    
}


fn reset_game_state (final_config : &parseconfig::MergedConfig, game_record : &mut GameState){
    game_record.trys.clear();
    game_record.days += 1;
    game_record.word  = {
        if final_config.random {
            game_record.final_set[game_record.days as usize].clone()
        }
        else{
            unreachable!("should not reach here");
        }
    }
}

fn generate_game_state( final_config: &parseconfig::MergedConfig) -> GameState {
    let mut ret_GS = GameState {
        word: String::new(),
        final_set: Vec::new(),
        acc_set: Vec::new(),
        days: 1,
        trys: Vec::new(),
    };

    ret_GS.final_set = read_voc_list(&(final_config.final_set_path)).expect("Failed to read final set");
    ret_GS.acc_set = read_voc_list(&(final_config.acc_set_path)).expect("Failed to read acceptable set");
    
    // words store must be sorted & capitalized
    for item in &mut ret_GS.final_set{
        *item = item.to_uppercase();
    }
    ret_GS.final_set.sort();

    for item in &mut ret_GS.acc_set{
        *item = item.to_uppercase();
    }
    ret_GS.acc_set.sort();

    // word list should be unique, 5-letter-long, final contained in acc
    if !check_voc_lists(&ret_GS.final_set, &ret_GS.acc_set) {
        panic!("Error: wordlist invalid.");
    }

    // in rand mode use shuffle & select 1-n in nth day
    if final_config.random {
        let mut rng = StdRng::seed_from_u64(final_config.seed.unwrap_or(20220123));
        ret_GS.final_set.shuffle(&mut rng);
        // println!("Shuffled final set: {:?}", ret_GS.final_set);
        ret_GS.days = final_config.day.unwrap_or(1);
        ret_GS.word =  ret_GS.final_set[ret_GS.days as usize].clone();  
    }
    else{
        ret_GS.days = 1;
        ret_GS.word = final_config.given_word.clone().expect("UNREACHABLE : In fixed mode, word is ensured to be given");
    }
    ret_GS
}



struct GameState {
    word : String,

    final_set : Vec<String>,
    acc_set : Vec<String>,
    days : u32,

    trys : Vec<(String,[Color;5])>,
}

fn read_voc_list(file_path : &str) -> Option<Vec<String>> {

    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}: {}", file_path, e);
            return None;
        }
    };
    
    let reader = BufReader::new(file);
    match reader.lines().collect::<Result<Vec<String>,_>>() {
        Ok(lines) => {
            return Some(lines)
        },
        Err(e) => {
            eprintln!("Error reading lines: {}", e);
            return None
        }
    }
   
   
}

fn check_voc_lists(final_set : &Vec<String>, acc_set : &Vec<String>) -> bool {

    let mut unique_property_acc = true;
    let mut hash_acc = HashSet::new();
    for item in acc_set {
        if !hash_acc.insert(item) {
            unique_property_acc = false; 
            break;
        }
    }
    if !unique_property_acc {
        eprintln!("Error: acceptable set contains duplicate words.");
        return false;
    }

    let mut unique_property_final = true;
    let mut hash_final = HashSet::new();
    for item in final_set {
        if !hash_final.insert(item) {
            unique_property_final = false; 
            break;
        }
    }
    if !unique_property_final {
        eprintln!("Error: acceptable set contains duplicate words.");
        return false;
    }

    let contain_property = final_set.iter().all(|x| hash_acc.contains(x));
    if !contain_property {
        eprintln!("Error: acceptable set does not contain all words in final set.");
        return false;
    }

    let lenth_property = acc_set.iter().all(|x| x.len() == 5);
    if !lenth_property {
        eprintln!("Error: acceptable set contains words that are not 5 letters long.");
        return false;
    }

    return true;
}


#[derive(PartialEq)]
enum Color{
    RED,
    YELLOW,
    GREEN,
    GREY,
}

impl Color{
    fn test_print(&self){
        match self {
            Color::RED => print!("R"),
            Color::YELLOW => print!("Y"),
            Color::GREEN => print!("G"),
            Color::GREY => print!("X"),
        }
    }

    fn colored_print(&self,letter : char){
        match self {
            Color::RED => print!("{}",letter.to_string().red()),
            Color::YELLOW => print!("{}",letter.to_string().yellow()),
            Color::GREEN => print!("{}",letter.to_string().green()),
            Color::GREY => print!("{}",letter.to_string().bright_black()),
        }
    }
}


fn check_word(ans : &str, guess: &str) -> [Color;5] {
    let mut green : [bool ; 5] = [false,false,false,false,false];
    for i in 0..5{
        green[i] = ans.chars().nth(i) == guess.chars().nth(i);
    }
    
    let mut yellow : [bool ; 5] = [false,false,false,false,false];
    let mut ans_yellow : [bool ; 5] = [false,false,false,false,false];
    for i in 0..5{
        for j in 0..5{
            yellow[i] = !green[i] && !green[j] && !ans_yellow[j] && ans.chars().nth(j) == guess.chars().nth(i);
            if yellow[i]{
                ans_yellow[j] = true;
                break;
            }
        }
    }

    let mut ret : [Color ; 5] = [Color::GREY,Color::GREY,Color::GREY,Color::GREY,Color::GREY];
    for i in 0..5{
        if yellow[i] { 
            ret[i] = Color::YELLOW; 
            //print!("Y"); 
        }
        else if green[i] { 
            ret[i] = Color::GREEN; 
            //print!("G"); 
        }
        else {
            ret[i] = Color::RED; 
            //print!("R"); 
        }
    }
    ret
}

fn check_valid_guess(guess : String,game_info : &GameState) -> bool{
    let contain_property = game_info.acc_set.contains(&guess);
    let lenth_property = guess.len() == 5;
    return contain_property && lenth_property;
}

fn check_valid_guess_difficult(guess : String,game_info : &GameState) -> bool{
    let guess_history = &game_info.trys;

    
    for item in guess_history{
        let mut mark : [bool ; 5] = [false,false,false,false,false]; // initialized every try
        for i in 0..5{
            if item.1[i] == Color::GREEN{ // green letters must appear in the same position
                if item.0.chars().nth(i) == guess.chars().nth(i){
                    mark[i] = true;
                }
                else{
                    return false;
                }
            }

            else if item.1[i] == Color::YELLOW{ // yellow letters must appear in the word
                let mut appear : bool = false;
                for j in 0..5{
                    if item.0.chars().nth(i) == guess.chars().nth(j) && !mark[j]{
                        mark[j] = true;
                        appear = true;
                        break;
                    }
                }
                if !appear{
                    return false;
                }
            }
        }
    }
    return true;
}


fn game_round(config_info : &parseconfig::MergedConfig , game_info : &GameState, alphabet : &mut HashMap<char,Color>) -> (String,[Color;5]) {

    let mut new_guess = String::new();
    let mut trimmed_guess;
    loop{
        new_guess.clear();
        io::stdin().read_line(&mut new_guess).expect("IO Error");
        trimmed_guess = new_guess.trim_end();
        let checker = match config_info.difficult{
            true => check_valid_guess_difficult,
            false => check_valid_guess,
        };
        if checker(trimmed_guess.to_string(),game_info){
            break;
        }
        else{
            println!("INVALID");
        }
    }

    let color_info = check_word(&game_info.word,trimmed_guess);

    for i in 0..5{
        let letter = new_guess.chars().nth(i).expect("UNREACHABLE");
        let color = &color_info[i];
        if *color == Color::GREEN{
            alphabet.insert(letter.clone(),Color::GREEN);
        }
        else if *color == Color::YELLOW && alphabet[&letter] != Color::GREEN{
            alphabet.insert(letter.clone(),Color::YELLOW);
        }
        else if *color == Color::RED && alphabet[&letter] != Color::GREEN && alphabet[&letter] != Color::YELLOW{
            alphabet.insert(letter.clone(),Color::RED);
        }
    }
    
    if config_info.is_tty{
        for i in 0..5{
            color_info[i].test_print();
        }
        println!("");

        for letter in 'A'..='Z'{
            alphabet[&letter].test_print();
        }
        println!("");
    }
    else {
        for i in 0..5{
            color_info[i].colored_print(new_guess.chars().nth(i).expect("UNREACHABLE"));
        }
        print!(" ");
        for letter in 'A'..='Z'{
            alphabet[&letter].colored_print(letter);
        }
        println!("");
    }

    (trimmed_guess.to_string(),color_info)
}

fn muilti_round(config_info : &parseconfig::MergedConfig , game_info : &mut GameState){
    let mut win_flag : bool = false;

    let mut alphabet : HashMap<char,Color> = HashMap::<char,Color>::new();
    for letter in 'A'..='Z'{
        alphabet.insert(letter,Color::GREY);
    }

    for i in 0..6{
        let round_result = game_round(config_info,game_info,&mut alphabet);
        win_flag = round_result.0 == game_info.word;
        game_info.trys.push(round_result);
        if win_flag {
            break;
        }
    }
    
    if win_flag{
        println!("CORRECT {}",game_info.trys.len());
    }
    else {
        println!("FAILED {}",game_info.word);
    }

    game_info.days += 1;

}

/*
{
  "total_rounds": 1,
  "games": [ 
    {
      "answer": "PROXY",
      "guesses": ["CRANE", "PROUD", "PROXY"]
    }
  ]
}
*/

#[derive(Debug, Serialize, Deserialize)]
struct JSONAux {
    answer: Option<String>,
    guesses: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JSONState {
    total_rounds: Option<u32>,
    games: Vec<JSONAux>,
}


impl JSONState {
    fn get_state_form_JSON(JSON_path : &str) -> Option<JSONState> {
        let data = match fs::read_to_string(JSON_path) {
            Ok(s) => s,
            Err(_) => {
                println!("No existing state file found, starting a new game.");
                return None;
            }
        };
        let cfg: JSONState = serde_json::from_str(&data).expect("JSON was not well-formatted and cannot be parsed");
        // println!("解析结果: {:?}", cfg);
        Some(cfg)
    }

    fn update_JSONstate(&mut self, new_game_state : & GameState){
        self.total_rounds = Some(self.total_rounds.unwrap_or(1) + 1);
        let new_record = JSONAux{
            answer : Some(new_game_state.word.clone()),
            guesses : {
                let mut temp = Vec::new();
                for item in &new_game_state.trys{
                    // print!("{}",item.0);
                    temp.push(item.0.clone());
                }
                Some(temp)
            }
        };
        
        
        self.games.push(new_record);
    }

    fn write_to_JSON(&mut self,write_path:&str) -> Option<()> {
        let json_str = serde_json::to_string_pretty(&self).ok()?;
        // println!("解析结果: {:?}", json_str);
        let mut file = File::create(write_path).ok()?;
        file.write_all(json_str.as_bytes()).ok()?;
        Some(())
    }

    fn stat(&self) -> StatData {
        let mut win_games = 0;
        let mut word_count : Vec<WordCount> = Vec::new();
        let mut total_guess = 0;

        for game in &self.games{ // for each game
            if let Some(answer) = &game.answer{
                if let Some(guesses) = &game.guesses{ // if answer & guesses is not None
                    for guess in guesses{  //for every guess
                        let mut found = false;
                        for record in &mut word_count{
                            if record.word == *guess{
                                record.cnt += 1;
                                found = true;
                                break;
                            }
                        }

                        if !found{
                            word_count.push(WordCount{ word : guess.to_string(), cnt : 1 });
                        }

                        if guess == answer {
                            win_games += 1;
                            total_guess += guesses.len();
                        }
                    }
                }   
            }
        }

        let win_rate : f32 = (win_games as f32) / (self.total_rounds.expect("UNREACHABLE") as f32);
        let avg_trys : f32 = (total_guess as f32) / (win_games as f32);
        word_count.sort();

        let ret = StatData {
            win_rate : win_rate,
            avg_trys : avg_trys,
            freq_words : {
                let mut tmp : [Option<WordCount> ; 5] = [None,None,None,None,None];
                for i in 0..5{
                    if i < word_count.len(){
                        tmp[i] = Some(word_count[i].clone());
                    }
                }
                tmp
            }
        };

        ret


    
    }
}

struct StatData{
    win_rate : f32,
    avg_trys : f32,
    freq_words : [Option<WordCount> ; 5],
}

impl StatData{
    fn print(&self){
        println!("Win Rate : {}",self.win_rate);
        println!("Avergae Guess : {}",self.avg_trys);
        for i in 0..5{
            if let Some(word_count) = &self.freq_words[i]{
                println!("{} th : {} , {} times", i + 1 ,word_count.word,word_count.cnt);
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct WordCount{
    word : String,
    cnt : u32,
}


impl Ord for WordCount{
    fn cmp(&self, other : &WordCount) -> std::cmp::Ordering{
        if self.cnt != other.cnt{
            return self.cnt.cmp(&other.cnt).reverse();
        }
        else{
            return self.word.cmp(&other.word);
        }
    }
}

impl PartialOrd for WordCount {
    fn partial_cmp(&self, other: &WordCount) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


// TODOs :
// --word consistency
// -tty test