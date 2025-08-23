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


fn main() -> () {
    let final_config = parseconfig::parse_config();

    let mut game_state = generate_game_state(&final_config);
    let JSON_path = &final_config.state_path;
    if let Some(JSON_path) = JSON_path {
        let state_from_JSON = JSONState::get_state_form_JSON(&JSON_path);
        merge_state(&mut game_state,&state_from_JSON);
    }
    // write_to_JSON();
     game_round(&final_config,&game_state);
    // check_word("BBBAA","AAABB");
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
}


fn merge_state(game_state : &mut GameState, json_state : &Option<JSONState>) -> () {
    if let Some(json_state) = json_state {
        if let Some(total_rounds) = json_state.total_rounds {
            game_state.trys = total_rounds;
        }
        if !json_state.games.is_empty() { // game vector is not empty, so that we can load the last game
            if let Some(last_game) = json_state.games.last() {
                if let Some(guesses) = &last_game.guesses {
                    game_state.guesses = guesses.clone();
                }
                if let Some(answer) = &last_game.answer {
                    game_state.word = answer.clone();
                }
            }
        }
    }
}

fn generate_game_state( final_config: &parseconfig::MergedConfig) -> GameState {
    let mut ret_GS = GameState {
        word: String::new(),
        guesses: Vec::new(),
        final_set: Vec::new(),
        acc_set: Vec::new(),
        days: 1,
        trys: 1,
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
    ret_GS.trys = 1;

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
    guesses : Vec<String>,

    final_set : Vec<String>,
    acc_set : Vec<String>,
    trys : u32,

    days : u32,
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


fn write_to_JSON(recorded_state : &mut JSONState,state_unwritten :GameState) -> Option<()> {

    recorded_state.total_rounds = 
        match recorded_state.total_rounds{
            None => Some(1),
            Some(n) => Some(n+1)
        };

    let new_game_state = JSONAux{
        answer : Some(state_unwritten.word),
        guesses : Some(state_unwritten.guesses)
    };

    recorded_state.games.push(new_game_state);


    let json_str = serde_json::to_string_pretty(&recorded_state).unwrap();
    let mut file = File::create("state_write.json").ok()?;
    file.write_all(json_str.as_bytes()).ok()?;
    Some(())
}

enum Color{
    RED,
    YELLOW,
    GREEN,
    GREY,
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
            print!("Y"); 
        }
        else if green[i] { 
            ret[i] = Color::GREEN; 
            print!("G"); 
        }
        else {
            ret[i] = Color::RED; 
            print!("R"); 
        }
    }
    ret
}

fn check_valid_guess(guess : String,game_info : &GameState) -> bool{
    let contain_property = game_info.acc_set.contains(&guess);
    let lenth_property = guess.len() == 5;
    return contain_property && lenth_property;
}


fn game_round(config_info : &parseconfig::MergedConfig , game_info : &GameState){

    let mut new_guess = String::new();
    let mut trimmed_guess;
    loop{
        io::stdin().read_line(&mut new_guess).expect("IO Error");
        trimmed_guess = new_guess.trim_end();
        if check_valid_guess(trimmed_guess.to_string(),game_info){
            break;
        }
        else{
            print!("INVALID");
        }
    }
    
    let color_info = check_word(game_info.word,trimmed_guess);
    if config_info.is_tty{
        
    }


}