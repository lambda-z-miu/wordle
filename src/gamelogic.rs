use crate::common;

use common::Color;
use common::GameState;
use common::MergedConfig;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};

use colored::*;
use std::collections::HashMap;
use std::io;

pub fn pure_game(final_config: MergedConfig, game_state_arg: GameState) -> () {
    let mut game_state = game_state_arg;

    let json_path = &final_config.state_path;
    let mut state_from_json: JSONState = JSONState {
        total_rounds: None,
        games: Vec::new(),
    }; // set a default empty state

    let mut opt_path = "tmp.json";
    if let Some(json_path) = json_path {
        if let Some(state) = JSONState::get_state_form_json(&json_path) {
            state_from_json = state;
            opt_path = "state_mem.json";
        }
    }

    if final_config.random {
        loop {
            muilti_round(&final_config, &mut game_state);
            state_from_json.update_jsonstate(&game_state);
            state_from_json.write_to_json(opt_path);
            if final_config.stats {
                let stat_data = state_from_json.stat();
                stat_data.print();
            }
            println!("insert q to end");
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("unable to read");

            if opt_path == "tmp.json" {
                fs::remove_file("tmp.json").expect("Unable to delete tmp");
            }

            let trimmed_input = input.trim_end();
            if trimmed_input == 'q'.to_string() {
                break;
            }
            reset_game_state(&final_config, &mut game_state);
        }
    } else {
        muilti_round(&final_config, &mut game_state);
        state_from_json.update_jsonstate(&game_state);
        state_from_json.write_to_json("state_write.json");
        if final_config.stats {
            let stat_data = state_from_json.stat();
            stat_data.print();
        }
    }
    return;
}

pub fn reset_game_state(final_config: &MergedConfig, game_record: &mut GameState) {
    game_record.trys.clear();
    game_record.days += 1;
    game_record.word = {
        if final_config.random {
            game_record.final_set[game_record.days as usize].clone()
        } else {
            unreachable!("Game cannot continue when answer is specified");
        }
    };
    game_record.alphabet = {
        let mut ret: HashMap<char, Color> = HashMap::new();
        for letter in 'A'..='Z' {
            ret.insert(letter, Color::GREY);
        }
        ret
    };
}

pub fn generate_game_state(final_config: &MergedConfig) -> GameState {
    let mut ret_gs = GameState {
        word: String::new(),
        final_set: Vec::new(),
        acc_set: Vec::new(),
        days: 1,
        trys: Vec::new(),
        alphabet: {
            let mut ret: HashMap<char, Color> = HashMap::new();
            for letter in 'A'..='Z' {
                ret.insert(letter, Color::GREY);
            }
            ret
        },
    };

    ret_gs.final_set =
        read_voc_list(&(final_config.final_set_path)).expect("Failed to read final set");
    ret_gs.acc_set =
        read_voc_list(&(final_config.acc_set_path)).expect("Failed to read acceptable set");

    // words store must be sorted & capitalized
    for item in &mut ret_gs.final_set {
        *item = item.to_uppercase();
    }
    ret_gs.final_set.sort();

    for item in &mut ret_gs.acc_set {
        *item = item.to_uppercase();
    }
    ret_gs.acc_set.sort();

    // word list should be unique, 5-letter-long, final contained in acc
    if !check_voc_lists(&ret_gs.final_set, &ret_gs.acc_set) {
        panic!("Error: wordlist invalid.");
    }

    // in rand mode use shuffle & select 1-n in nth day
    if final_config.random {
        let mut rng = StdRng::seed_from_u64(final_config.seed.unwrap_or(20220123));
        ret_gs.final_set.shuffle(&mut rng);
        // println!("Shuffled final set: {:?}", ret_gs.final_set);
        ret_gs.days = final_config.day.unwrap_or(1);
        ret_gs.word = ret_gs.final_set[ret_gs.days as usize].clone();
    } else {
        ret_gs.days = 1;
        ret_gs.word = final_config
            .given_word
            .clone()
            .expect("UNREACHABLE : In fixed mode, word is ensured to be given");
    }
    ret_gs
}

fn read_voc_list(file_path: &str) -> Option<Vec<String>> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}: {}", file_path, e);
            return None;
        }
    };

    let reader = BufReader::new(file);
    match reader.lines().collect::<Result<Vec<String>, _>>() {
        Ok(lines) => return Some(lines),
        Err(e) => {
            eprintln!("Error reading lines: {}", e);
            return None;
        }
    }
}

fn check_voc_lists(final_set: &Vec<String>, acc_set: &Vec<String>) -> bool {
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

impl Color {
    fn colored_print(&self, letter: char) {
        match self {
            Color::RED => print!("{}", letter.to_string().red()),
            Color::YELLOW => print!("{}", letter.to_string().yellow()),
            Color::GREEN => print!("{}", letter.to_string().green()),
            Color::GREY => print!("{}", letter.to_string().bright_black()),
        }
    }
}

pub fn check_word(ans: &str, guess: &str) -> [Color; 5] {
    let mut green: [bool; 5] = [false;5];
    for i in 0..5 {
        green[i] = ans.chars().nth(i) == guess.chars().nth(i);
    }

    let mut yellow: [bool; 5] = [false;5];
    let mut ans_yellow: [bool; 5] = [false;5];
    for i in 0..5 {
        for j in 0..5 {
            yellow[i] = !green[i]
                && !green[j]
                && !ans_yellow[j]
                && ans.chars().nth(j) == guess.chars().nth(i);
            if yellow[i] {
                ans_yellow[j] = true;
                break;
            }
        }
    }

    let mut ret: [Color; 5] = [
        Color::GREY,
        Color::GREY,
        Color::GREY,
        Color::GREY,
        Color::GREY,
    ];
    for i in 0..5 {
        if yellow[i] {
            ret[i] = Color::YELLOW;
            //print!("Y");
        } else if green[i] {
            ret[i] = Color::GREEN;
            //print!("G");
        } else {
            ret[i] = Color::RED;
            //print!("R");
        }
    }
    ret
}

pub fn check_valid_guess(guess: String, game_info: &GameState) -> bool {
    let contain_property = game_info.acc_set.contains(&guess);
    let lenth_property = guess.len() == 5;
    return contain_property && lenth_property;
}

pub fn check_valid_guess_difficult(guess: String, game_info: &GameState) -> bool {
    let guess_history = &game_info.trys;

    for item in guess_history {
        let mut mark: [bool; 5] = [false, false, false, false, false]; // initialized every try
        for i in 0..5 {
            if item.1[i] == Color::GREEN {
                // green letters must appear in the same position
                if item.0.chars().nth(i) == guess.chars().nth(i) {
                    mark[i] = true;
                } else {
                    return false;
                }
            } else if item.1[i] == Color::YELLOW {
                // yellow letters must appear in the word
                let mut appear: bool = false;
                for j in 0..5 {
                    if item.0.chars().nth(i) == guess.chars().nth(j) && !mark[j] {
                        mark[j] = true;
                        appear = true;
                        break;
                    }
                }
                if !appear {
                    return false;
                }
            }
        }
    }
    return true;
}

fn input_guess(config_info: &MergedConfig, game_info: &GameState) -> String {
    let mut new_guess = String::new();
    let mut trimmed_guess;
    loop {
        new_guess.clear();
        io::stdin().read_line(&mut new_guess).expect("IO Error");
        trimmed_guess = new_guess.trim_end().to_string();
        let checker = match config_info.difficult {
            true => check_valid_guess_difficult,
            false => check_valid_guess,
        };
        if checker(trimmed_guess.to_string(), game_info) {
            break;
        } else {
            println!("INVALID");
        }
    }
    return trimmed_guess;
}

pub fn paint_keyboad(game_info: &mut GameState, color_info: [Color; 5], guess_arg: &str) {
    for i in 0..5 {
        let letter = guess_arg.chars().nth(i).expect("UNREACHABLE");
        let color = &color_info[i];
        if *color == Color::GREEN {
            game_info.alphabet.insert(letter.clone(), Color::GREEN);
        } else if *color == Color::YELLOW && game_info.alphabet[&letter] != Color::GREEN {
            game_info.alphabet.insert(letter.clone(), Color::YELLOW);
        } else if *color == Color::RED
            && game_info.alphabet[&letter] != Color::GREEN
            && game_info.alphabet[&letter] != Color::YELLOW
        {
            game_info.alphabet.insert(letter.clone(), Color::RED);
        }
    }
}

pub fn game_round(
    game_info: &mut GameState,
    guess_arg: String,
) -> (String, [Color; 5]) {
    let color_info = check_word(&game_info.word, &guess_arg);
    paint_keyboad(game_info, color_info, &guess_arg);
    (guess_arg.to_string(), color_info)
}

fn muilti_round(config_info: &MergedConfig, game_info: &mut GameState) {
    let mut win_flag: bool = false;

    for _i in 0..6 {
        let round_result = game_round(
            game_info,
            input_guess(&config_info, &game_info),
        );
        let color_info = &round_result.1;
        let guess_arg = &round_result.0;
        for i in 0..5 {
            color_info[i].colored_print(guess_arg.chars().nth(i).expect("UNREACHABLE"));
        }
        print!(" ");
        for letter in 'A'..='Z' {
            game_info.alphabet[&letter].colored_print(letter);
        }
        println!("");
        win_flag = round_result.0 == game_info.word;
        game_info.trys.push(round_result);
        if win_flag {
            break;
        }
    }

    if win_flag {
        println!("CORRECT {}", game_info.trys.len());
    } else {
        println!("FAILED {}", game_info.word);
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
    fn get_state_form_json(json_path: &str) -> Option<JSONState> {
        let data = match fs::read_to_string(json_path) {
            Ok(s) => s,
            Err(_) => {
                println!("No existing state file found, starting a new game.");
                return None;
            }
        };
        let cfg: JSONState =
            serde_json::from_str(&data).expect("JSON was not well-formatted and cannot be parsed");
        // println!("解析结果: {:?}", cfg);
        Some(cfg)
    }

    fn update_jsonstate(&mut self, new_game_state: &GameState) {
        self.total_rounds = Some(self.total_rounds.unwrap_or(1) + 1);
        let new_record = JSONAux {
            answer: Some(new_game_state.word.clone()),
            guesses: {
                let mut temp = Vec::new();
                for item in &new_game_state.trys {
                    // print!("{}",item.0);
                    temp.push(item.0.clone());
                }
                Some(temp)
            },
        };

        self.games.push(new_record);
    }

    fn write_to_json(&mut self, write_path: &str) -> Option<()> {
        let json_str = serde_json::to_string_pretty(&self).ok()?;
        // println!("解析结果: {:?}", json_str);
        let mut file = File::create(write_path).ok()?;
        file.write_all(json_str.as_bytes()).ok()?;
        Some(())
    }

    fn stat(&self) -> StatData {
        let mut win_games = 0;
        let mut word_count: Vec<WordCount> = Vec::new();
        let mut total_guess = 0;

        for game in &self.games {
            // for each game
            if let Some(answer) = &game.answer {
                if let Some(guesses) = &game.guesses {
                    // if answer & guesses is not None
                    for guess in guesses {
                        //for every guess
                        let mut found = false;
                        for record in &mut word_count {
                            if record.word == *guess {
                                record.cnt += 1;
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            word_count.push(WordCount {
                                word: guess.to_string(),
                                cnt: 1,
                            });
                        }

                        if guess == answer {
                            win_games += 1;
                            total_guess += guesses.len();
                        }
                    }
                }
            }
        }

        let win_rate: f32 = (win_games as f32) / (self.total_rounds.expect("UNREACHABLE") as f32);
        let avg_trys: f32 = (total_guess as f32) / (win_games as f32);
        word_count.sort();

        let ret = StatData {
            win_rate: win_rate,
            avg_trys: avg_trys,
            freq_words: {
                let mut tmp: [Option<WordCount>; 5] = [None, None, None, None, None];
                for i in 0..5 {
                    if i < word_count.len() {
                        tmp[i] = Some(word_count[i].clone());
                    }
                }
                tmp
            },
        };

        ret
    }
}

struct StatData {
    win_rate: f32,
    avg_trys: f32,
    freq_words: [Option<WordCount>; 5],
}

impl StatData {
    fn print(&self) {
        println!("Win Rate : {}", self.win_rate);
        println!("Avergae Guess : {}", self.avg_trys);
        for i in 0..5 {
            if let Some(word_count) = &self.freq_words[i] {
                println!(
                    "{} th : {} , {} times",
                    i + 1,
                    word_count.word,
                    word_count.cnt
                );
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct WordCount {
    word: String,
    cnt: u32,
}

impl Ord for WordCount {
    fn cmp(&self, other: &WordCount) -> std::cmp::Ordering {
        if self.cnt != other.cnt {
            return self.cnt.cmp(&other.cnt).reverse();
        } else {
            return self.word.cmp(&other.word);
        }
    }
}

impl PartialOrd for WordCount {
    fn partial_cmp(&self, other: &WordCount) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

