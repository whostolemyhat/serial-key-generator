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
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    num_bytes: i8,
    byte_shifts: Vec<(i16, i16, i16)>,
    hash: String,
    blacklist: Vec<String>
}

impl Config {
    fn new(hash: &str, num_bytes: i8, blacklist: Vec<String>) -> Config {
        let hash_str = hash.to_string();
        Config {
            num_bytes: num_bytes,
            byte_shifts: generate_shifts(num_bytes),
            hash: hash_str.clone(),
            blacklist
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

fn read_config_from_file(path: &Path) -> Result<Config, io::Error> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    let config: Config = serde_json::from_str(&s)?;
    Ok(config)
}

fn write_config(config_json: &str, path: &Path) -> Result<(), io::Error> {
    let mut f = File::create(path)?;
    f.write_all(&config_json.as_bytes())?;
    Ok(())
}

fn create_hash() -> String {
    let len = rand::thread_rng().gen_range(24, 32);
    rand::thread_rng()
        .gen_ascii_chars()
        .take(len)
        .collect()
}

fn main() {
    // black list is array of seeds
    // let blacklist = vec!["11111111"]; // 9227B6EA
    // let key = "3ABC-9099-E39D-4E65-E060"; len 4
    // TODO: create config and give to user eg bytes and transforms used, hash for crc32
    // TODO gen hash if none provided
    // TODO add blacklist command (save to config)
    // TODO: readme

    // TODO: flow: keygen myname@example.com => 1234-1234-1234-1234
    // keygen create -s SEED -h HASH -c CONFIG -b BLACKLIST -l LENGTH
    // keygen create -s jamestease@hotmail.com -h brian -b 11111111 -b 22222222 -l 32 -c config.json
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
                            .takes_value(true)
                            .required(true))
                        .arg(Arg::with_name("hash")
                            .help("String to add to the key to create a hash")
                            .short("h")
                            .long("hash")
                            .value_name("HASH")
                            .takes_value(true))
                        .arg(Arg::with_name("length")
                            .help("Number of bytes to create in key")
                            .short("l")
                            .long("length")
                            .takes_value(true)
                            .value_name("LENGTH"))
                        .arg(Arg::with_name("config")
                            .help("Existing config file to use to generate key")
                            .short("c")
                            .long("config")
                            .takes_value(true)
                            .value_name("CONFIG"))
                        .arg(Arg::with_name("output")
                            .help("Path to save config file")
                            .short("o")
                            .long("output")
                            .takes_value(true)
                            .value_name("OUTPUT"))
                        .arg(Arg::with_name("blacklist")
                            .help("Seed values to add to blacklist")
                            .short("b")
                            .long("blacklist")
                            .takes_value(true)
                            .multiple(true)
                            .value_name("BLACKLIST"))
                    )
                    .subcommand(SubCommand::with_name("verify")
                                .about("Check if key is valid")
                                .arg(Arg::with_name("key")
                                    .help("Key to check")
                                    .short("k")
                                    .long("key")
                                    .takes_value(true)
                                    .value_name("KEY")
                                    .required(true))
                                .arg(Arg::with_name("config")
                                    .help("Path to config file")
                                    .short("c")
                                    .long("config")
                                    .takes_value(true)
                                    .required(true)))
                    .subcommand(SubCommand::with_name("checksum")
                                .about("Check if key checksum is valid")
                                .arg(Arg::with_name("key")
                                    .help("Key to check")
                                    .short("k")
                                    .long("key")
                                    .takes_value(true)
                                    .required(true))
                                .arg(Arg::with_name("config")
                                    .help("Path to config file")
                                    .short("c")
                                    .long("config")
                                    .takes_value(true)
                                    .required(true)))
                    .get_matches();

    let verify = matches.subcommand_matches("verify");
    // TODO: read config from file
    match verify {
        Some(arg) => {
            let key = arg.value_of("key").unwrap(); // required so unwrap ok
            let path = arg.value_of("config").unwrap();
            match read_config_from_file(Path::new(&path)) {
                Ok(config) => {
                    println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
                },
                Err(e) => panic!("Error with config: {:?}", e)
            }
        },
        None => {}
    }

    let checksum = matches.subcommand_matches("checksum");
    match checksum {
        Some(arg) => {
            let key = arg.value_of("key").unwrap();
            let path = arg.value_of("config").unwrap();
            match read_config_from_file(Path::new(&path)) {
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
            let config = match arg.is_present("config") {
                true => {
                    // read existing config
                    let path = arg.value_of("config").unwrap();
                    match read_config_from_file(Path::new(path)) {
                        Ok(config) => config,
                        Err(e) => panic!("Error reading config: {:?}", e)
                    }
                },
                false => {
                    // create new config
                    let gen_hash = create_hash();
                    let hash = arg.value_of("hash").unwrap_or(&gen_hash[..]);
                    let len = arg.value_of("length").unwrap_or("8");
                    let num_bytes: i8 = len.parse().expect("Max bytes needs to be i8");
                    let mut blacklist: Vec<String> = vec![];
                    if arg.is_present("blacklist") {
                        // convert Vec<str> to Vec<String>
                        blacklist = arg.values_of("blacklist").unwrap().map(|s| s.to_string()).collect();
                    }
                    Config::new(hash, num_bytes, blacklist)
                }
            };

            let config_json = config_to_json(&config).expect("Error creating config");

            match arg.value_of("output") {
                Some(path) => {
                    match write_config(&config_json, Path::new(path)) {
                        Err(e) => println!("Error saving config: {:?}", e),
                        Ok(_) => println!("Config file saved at {}", path)
                    }
                },
                None => {
                    if !arg.is_present("config") {
                        println!("Save this configuration data! You will need it to validate keys.");
                        println!("{}", config_json);
                        println!("--");
                    }
                }
            }


            let input = arg.value_of("seed").unwrap();
            let seed = crc32::checksum_ieee(format!("{}+{}", input, &config.hash).as_bytes()) as i64;
            let key = make_key(&seed, &config.num_bytes, &config.byte_shifts);

            println!("{}", key);
            // println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
            // println!("{:?}", check_key_checksum(&key, &config.num_bytes));
        },
        None => {}
    };
}