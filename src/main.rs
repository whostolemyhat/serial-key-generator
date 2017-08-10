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

fn read_config_from_file() -> Result<Config, io::Error> {
    let mut s = String::new();
    File::open("config.json")?.read_to_string(&mut s)?;
    let config: Config = serde_json::from_str(&s)?;
    Ok(config)
}

fn write_config(config_json: &str) -> Result<(), io::Error> {
    let mut f = File::create("config.json")?;
    f.write_all(&config_json.as_bytes())?;
    Ok(())
}

fn main() {
    // black list is array of seeds
    // let blacklist = vec!["11111111"]; // 9227B6EA
    // let key = "3ABC-9099-E39D-4E65-E060"; len 4
    // TODO: create config and give to user eg bytes and transforms used, hash for crc32
    // TODO gen hash if none provided
    // TODO add blacklist command (save to config)

    // TODO: flow: keygen myname@example.com => 1234-1234-1234-1234
    // keygen create -s SEED -h HASH -c CONFIG -b BLACKLIST -l LENGTH
    // keygen verify 1234-1324-1234-1234 => Status
    // keygen checksum e096 => Status

    let matches = App::new("Keygen")
                    .version("1.0")
                    .author("James Tease <james@jamestease.co.uk>")
                    .about("Generates and verifies serial keys")
                    .subcommand(SubCommand::with_name("create")
                        .about("Create a new serial key")
                        .arg(Arg::with_name("seed")
                            .help("String used to create a key")
                            .short("s")
                            .long("seed")
                            .value_name("SEED")
                            .required(true))
                        .arg(Arg::with_name("HASH")
                             .help("String to add to the key to create a hash")
                             .short("h")
                             .long("hash")
                             .value_name("HASH")
                            .required(true)))
                    .subcommand(SubCommand::with_name("verify")
                                .about("Check if key is valid")
                                .arg(Arg::with_name("KEY")
                                        .required(true)))
                    .subcommand(SubCommand::with_name("checksum")
                                .about("Check if key checksum is valid")
                                .arg(Arg::with_name("KEY")
                                     .required(true))
                                .arg(Arg::with_name("CONFIG")
                                     .required(true)))
                    .get_matches();

    let verify = matches.subcommand_matches("verify");
    // TODO: read config from file
    match verify {
        Some(arg) => {
            let key = arg.value_of("KEY").unwrap(); // required so unwrap ok
            match read_config_from_file() {
                Ok(config) => {
                    println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
                },
                Err(e) => println!("Error with config: {:?}", e)
            }
        },
        None => {}
    }

    let checksum = matches.subcommand_matches("checksum");
    // TODO: read config from file
    match checksum {
        Some(arg) => {
            let key = arg.value_of("KEY").unwrap();
            match read_config_from_file() {
                Ok(config) => {
                    println!("{:?}", check_key_checksum(&key, &config.num_bytes));
                },
                Err(e) => println!("Error with config: {:?}", e)
            }
        },
        None => {}
    }

    let create = matches.subcommand_matches("create");
    match create {
        Some(arg) => {
            // TODO: take hash
            let hash = arg.value_of("hash").unwrap();
            let config = Config::new(hash);
            // TODO: unwrap
            let config_json = config_to_json(&config).unwrap();

            println!("Save this configuration data! You will need it to validate keys.");
            println!("{:?}", config_json);
            match write_config(&config_json) {
                Err(e) => println!("Error saving config: {:?}", e),
                Ok(_) => println!("Config file saved at config.json")
            }

            let input = arg.value_of("seed").unwrap();
            println!("inouyt {:?}", input);
            let seed = crc32::checksum_ieee(format!("{}+{}", input, &config.hash).as_bytes()) as i64;
            let key = make_key(&seed, &config.num_bytes, &config.byte_shifts);
            println!("{}", key);
            println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
        },
        None => {}
    };
}