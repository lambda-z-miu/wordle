mod parseconfig;
use rand::Rng;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashSet;
use rand::seq::SliceRandom; 
use rand::SeedableRng; 
use rand::rngs::StdRng;

fn main() -> () {
    let final_config = parseconfig::parse_config();
    let GameState = generate_game_state(&final_config);
}


fn generate_game_state( final_config: &parseconfig::MergedConfig) -> GameState {
    let mut ret_GS = GameState {
        word: String::new(),
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