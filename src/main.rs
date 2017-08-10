extern crate regex;
extern crate crc;
extern crate clap;
extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod keygen;

use clap::{ App, Arg, SubCommand };
use keygen::{ make_key, check_key, check_key_checksum };
use crc::{ crc32 };
use rand::Rng;
use serde_json::Error;
use std::io;
use std::io::{ Read, Write };
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    num_bytes: i8,
    byte_shifts: Vec<(i16, i16, i16)>,
    hash: String,
    blacklist: Vec<String>
}

impl Config {
    fn new(hash: &str) -> Config {
        let hash_str = hash.to_string();
        Config {
            num_bytes: 12,
            byte_shifts: generate_shifts(12),
            hash: hash_str.clone(),
            blacklist: vec!["11111111".to_string()]
        }
    }
}

fn generate_shifts(len: i8) -> Vec<(i16, i16, i16)> {
    let mut shifts: Vec<(i16, i16, i16)> = Vec::new();
    for _ in 0..len {
        shifts.push((rand::thread_rng().gen_range(0, 255), rand::thread_rng().gen_range(0, 255), rand::thread_rng().gen_range(0, 255)));
    }

    shifts
}

fn config_to_json(config: &Config) -> Result<String, Error> {
    let j = serde_json::to_string(&config)?;
    Ok(j)
}

fn read_config_from_file() -> Result<String, io::Error> {
    let mut s = String::new();
    File::open("config.json")?.read_to_string(&mut s)?;
    Ok(s)
}

fn write_config(config_json: &str) -> Result<(), io::Error> {
    let mut f = File::create("config.json")?;
    f.write_all(&config_json.as_bytes())?;
    Ok(())
}

fn main() {
    // black list is array of seeds
    // let blacklist = vec!["11111111"]; // 9227B6EA
    // let hash = "hello";
    // let key = "3ABC-9099-E39D-4E65-E060";
    // TODO: create config and give to user eg bytes and transforms used, hash for crc32
    // TODO: flow: keygen myname@example.com => 1234-1234-1234-1234
    // keygen verify 1234-1324-1234-1234 => Status
    // keygen checksum e096 => Status

    let matches = App::new("Keygen")
                    .version("1.0")
                    .author("James Tease <james@jamestease.co.uk>")
                    .about("Generates and verifies serial keys")
                    .arg(Arg::with_name("SEED")
                         .help("String used to create a key")
                        .required(true))
                    .arg(Arg::with_name("HASH")
                         .help("String to add to the key to create a hash")
                        .required(true))
                    .subcommand(SubCommand::with_name("verify")
                                .about("Check if key is valid")
                                .arg(Arg::with_name("KEY")
                                        .required(true))
                                .arg(Arg::with_name("CONFIG")
                                     .required(true)))
                    .subcommand(SubCommand::with_name("checksum")
                                .about("Check if key checksum is valid")
                                .arg(Arg::with_name("KEY")
                                     .required(true))
                                .arg(Arg::with_name("CONFIG")
                                     .required(true)))
                    .get_matches();

    // let verify = matches.subcommand_matches("verify");
    // TODO: read config from file
    // match verify {
    //     Some(arg) => {
    //         let key = arg.value_of("KEY").unwrap(); // required so unwrap ok
    //         let config = arg.value_of("CONFIG").unwrap();
    //         println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
    //     },
    //     None => {}
    // }

    // let checksum = matches.subcommand_matches("checksum");
    // TODO: read config from file
    // match checksum {
    //     Some(arg) => {
    //         let key = arg.value_of("KEY").unwrap();
    //         let config = arg.value_of("CONFIG").unwrap();
    //         println!("{:?}", check_key_checksum(&key, &config.num_bytes));
    //     },
    //     None => {}
    // }

    let create = matches.value_of("SEED");
    match create {
        Some(input) => {
            // TODO: take hash
            let config = Config::new("hello");
            // TODO: unwrap
            let config_json = config_to_json(&config).unwrap();

            // TODO: save to file
            println!("Save this configuration data! You will need it to validate keys.");
            println!("{:?}", config_json);
            match write_config(&config_json) {
                Err(e) => println!("Error saving config: {:?}", e),
                Ok(_) => println!("Config file saved at config.json")
            }

            let seed = crc32::checksum_ieee(format!("{}+{}", input, &config.hash).as_bytes()) as i64;
            let key = make_key(&seed, &config.num_bytes, &config.byte_shifts);
            println!("{}", key);
            println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
        },
        None => {}
    };
}