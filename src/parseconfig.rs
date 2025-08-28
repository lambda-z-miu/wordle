use clap::ArgGroup;
use clap::Parser;
use serde::Deserialize;
use std::fs;

use crate::common;
use common::MergedConfig;

const DEFAULT_FINAL_SET: &str = "words.txt";
const DEFAULT_ACC_SET: &str = "words.txt";

/*
  "random": true,
  "difficult": false,
  "stats": true,
  "day": 5,
  "seed": 20220123,
  "final_set": "fin.txt",
  "acceptable_set": "acc.txt",
  "state": "state.json",
  "word": "cargo"
*/

#[derive(Debug, Deserialize)]
struct JsonConfig {
    random: Option<bool>,
    difficult: Option<bool>,
    stats: Option<bool>,
    day: Option<u32>,
    seed: Option<u64>,
    final_set: Option<String>,
    acceptable_set: Option<String>,
    state: Option<String>,
    word: Option<String>,
}

impl JsonConfig {
    fn get_config_form_json(json_path: &String) -> JsonConfig {
        let data = fs::read_to_string(json_path).expect("Unable to read JSON config");
        let cfg: JsonConfig = serde_json::from_str(&data).expect("JSON was not well-formatted");
        // println!("解析结果: {:?}", cfg);
        cfg
    }

    fn check_json_config(&self) -> bool {
        let day_valid = self.day.is_some();
        let stat_valid = self.stats.is_some();
        let seed_valid = self.seed.is_some();
        let word_valid = self.word.is_some();
        let check_random = day_valid || stat_valid || seed_valid;
        let check_random_word = !word_valid;
        if check_random != check_random_word {
            return false;
        }
        if let Some(json_random) = self.random {
            if json_random != check_random {
                return false;
            }
        }
        return true;
    }
}

struct CmdConfig {
    random: bool,
    difficult: bool,
    stats: bool,
    day: Option<u32>,
    seed: Option<u64>,
    final_set_path: Option<String>,
    acc_set_path: Option<String>,
    state_path: Option<String>,
    given_word: Option<String>,
    config_path: Option<String>,
    ui: bool,
}

impl CmdConfig {
    fn convert_cmd_to_merged(&self) -> MergedConfig {
        MergedConfig {
            random: self.random,
            difficult: self.difficult,
            stats: self.stats,
            day: self.day,
            seed: self.seed,
            final_set_path: self
                .final_set_path
                .clone()
                .unwrap_or(DEFAULT_FINAL_SET.to_string()),
            acc_set_path: self
                .acc_set_path
                .clone()
                .unwrap_or(DEFAULT_ACC_SET.to_string()),
            state_path: self.state_path.clone(),
            given_word: self.given_word.clone(),
            ui: self.ui,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    group(
        ArgGroup::new("choice")
            .args(&["random", "word"])
            .required(true)
            .multiple(false)
    )
)]
#[command(name = "wordle", version = "1.0", about = "Wordle")]
struct Args {

    #[arg(short = 'w', long = "word")]
    word: Option<String>,

    #[arg(short = 'd', long = "day", requires = "random")]
    day: Option<u32>,

    #[arg(short = 's', long = "seed", requires = "random")]
    seed: Option<u64>,

    #[arg(short = 't', long = "stats", requires = "random")]
    stats: bool,

    #[arg(short = 'f', long = "final-set")]
    final_set_path: Option<String>,

    #[arg(short = 'a', long = "acceptable-set")]
    acc_set_path: Option<String>,

    #[arg(short = 'D', long = "difficult")]
    difficult: bool,

    #[arg(short = 'S', long = "state")]
    state_path: Option<String>,

    #[arg(short = 'c', long = "config")]
    config_path: Option<String>,

    #[arg(short = 'r', long = "random")]
    random: bool,

    #[arg(short = 'u', long = "gui")]
    ui: bool,
}

impl Args {
    fn parse_cmd_args(self) -> CmdConfig {
        let mut ret_gc = CmdConfig {
            random: self.random,
            difficult: self.difficult,
            day: self.day,
            seed: self.seed,
            stats: self.stats,
            given_word: self.word,
            final_set_path: self.final_set_path,
            acc_set_path: self.acc_set_path,
            state_path: self.state_path,
            config_path: self.config_path,
            ui: self.ui,
        };
        ret_gc.random = ret_gc.given_word.is_none();

        return ret_gc;
    }
}

fn merge_config(cmd_config: &CmdConfig, json_config: &JsonConfig) -> MergedConfig {
    let mut ret_mc = MergedConfig {
        random: cmd_config.random,
        difficult: cmd_config.difficult,
        stats: false,
        day: None,
        seed: None,
        final_set_path: String::new(),
        acc_set_path: String::new(),
        state_path: cmd_config.state_path.clone(),
        given_word: cmd_config.given_word.clone(),
        ui: cmd_config.ui,
    };

    if !cmd_config.random {
        ret_mc.random = json_config.random.unwrap_or(false);
    } // random canbe set by JSON or cmd but cmd args have higher priority

    if !cmd_config.difficult {
        ret_mc.difficult = json_config.difficult.unwrap_or(false);
    } // difficult canbe set by JSON or cmd IN ANY MODE

    if ret_mc.random {
        if !cmd_config.stats {
            ret_mc.stats = json_config.stats.unwrap_or(false);
        } // stats canbe set by JSON or cmd ONLY IN RANDOM MODE, cmd args have higher priority
        if cmd_config.day.is_none() {
            ret_mc.day = json_config.day;
        } // day canbe set by JSON or cmd ONLY IN RANDOM MODE, cmd args have higher priority
        if cmd_config.seed.is_none() {
            ret_mc.seed = json_config.seed;
        } // seed canbe set by JSON or cmd ONLY IN RANDOM MODE, cmd args have higher priority
        if json_config.word.is_some() {
            panic!("Incosistent JSON config: word can NOT be set in random mode");
        } // word can NOT be set in random mode
    } else {
        // in fixed mode, stats, day, seed are ensured not to be set in cmd args
        if json_config.stats.is_some() {
            panic!("Incosistent JSON config: stat can NOT be set in fixed mode");
        } // stats can NOT be set in fixed mode
        if json_config.day.is_some() {
            panic!("Incosistent JSON config: day can NOT be set in fixed mode");
        } // day can NOT be set in fixed mode
        if json_config.seed.is_some() {
            panic!("Incosistent JSON config: seed can NOT be set in fixed mode");
        } // seed can NOT be set in fixed mode
        // we require that if random is false, then word must be set in cmd args, thus no need to merge word from JSON
    }

    if cmd_config.final_set_path.is_none() {
        ret_mc.final_set_path = json_config
            .final_set
            .clone()
            .unwrap_or(DEFAULT_FINAL_SET.to_string());
    } // final_set canbe set by JSON or cmd IN ANY MODE, cmd args have higher priority
    if cmd_config.acc_set_path.is_none() {
        ret_mc.acc_set_path = json_config
            .acceptable_set
            .clone()
            .unwrap_or(DEFAULT_ACC_SET.to_string());
    } // acceptable_set canbe set by JSON or cmd IN ANY MODE, cmd args have higher priority
    if cmd_config.state_path.is_none() {
        ret_mc.state_path = json_config.state.clone();
    } // acceptable_set canbe set by JSON or cmd IN ANY MODE, cmd args have higher priority

    // Config state_path is no longer used, thus we do not merge it
    ret_mc
}

pub fn parse_config() -> MergedConfig {
    let args: Args = Args::parse();
    let cmd_config = args.parse_cmd_args();

    let from_json: JsonConfig;
    let final_config: MergedConfig;

    final_config = match &cmd_config.config_path {
        Some(path) => {
            from_json = JsonConfig::get_config_form_json(&path);
            if !from_json.check_json_config() {
                panic!("JSON config is not valid, please check the config file.");
            }
            merge_config(&cmd_config, &from_json) // final config generated and checked
        }
        None => cmd_config.convert_cmd_to_merged(),
    };
    final_config
}
