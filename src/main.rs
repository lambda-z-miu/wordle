mod parseconfig;
use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashSet;
use rand::seq::SliceRandom; 
use rand::SeedableRng; 
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use serde_json;

fn main() -> () {
    let final_config = parseconfig::parse_config();
    let mut game_state = generate_game_state(&final_config);
    let JSON_path = final_config.state_path;
    if let Some(JSON_path) = JSON_path {
        let state_from_JSON = JSONState::get_state_form_JSON(&JSON_path);
        merge_state(&mut game_state,&state_from_JSON);
    }

}


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
    if !check_voc_lists(&ret_GS.final_set, &ret_GS.acc_set) {
        panic!("Error: wordlist invalid.");
    }
    ret_GS.trys = 1;

    if final_config.random {
        let mut rng = StdRng::seed_from_u64(final_config.seed.unwrap_or(20220123));
        ret_GS.final_set.shuffle(&mut rng);
        // println!("Shuffled final set: {:?}", ret_GS.final_set);
        ret_GS.days = final_config.day.unwrap_or(1);   
    }
    else{
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

fn game_round(game_state : & GameState, final_config : & parseconfig::MergedConfig) -> () {
    
}

/* 

fn get_random_word( words : &mut Vec<String>) -> String {
    let lenth = words.len();
    let num = rand::rng().random_range(0.. lenth);
    words.remove(num);
    words[num].clone()
}
*/