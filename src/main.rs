extern crate regex;
extern crate crc;
extern crate clap;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate serial_key;

use clap::{ App, Arg, SubCommand };
use serial_key::{ make_key, check_key, check_key_checksum };
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

#[derive(Serialize, Deserialize)]
pub struct Output {
    key: String,
    config: Config
}

impl Config {
    fn new(hash: &str, num_bytes: i8, blacklist: Vec<String>) -> Config {
        let hash_str = hash.to_string();
        Config {
            num_bytes: num_bytes,
            byte_shifts: Self::generate_shifts(num_bytes),
            hash: hash_str.clone(),
            blacklist
        }
    }

    pub fn config_to_json(&self) -> Result<String, Error> {
        let j = serde_json::to_string(&self)?;
        Ok(j)
    }

    fn generate_shifts(len: i8) -> Vec<(i16, i16, i16)> {
        let mut shifts: Vec<(i16, i16, i16)> = Vec::new();
        for _ in 0..len {
            shifts.push((rand::thread_rng().gen_range(0, 255), rand::thread_rng().gen_range(0, 255), rand::thread_rng().gen_range(0, 255)));
        }

        shifts
    }
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
    let matches = App::new("Keygen")
                    .version("1.3")
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
                        .arg(Arg::with_name("json")
                            .help("Format output as JSON")
                            .short("j")
                            .long("json"))
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
                                    .required_unless("length"))
                                .arg(Arg::with_name("length")
                                     .help("Number of bytes expected in key")
                                     .short("l")
                                     .long("length")
                                     .takes_value(true)
                                     .required_unless("config"))
                                )
                    .get_matches();

    match matches.subcommand_name() {
        Some("verify") => {
            if let Some(ref matches) = matches.subcommand_matches("verify") {
                let key = matches.value_of("key").unwrap(); // required so unwrap ok
                let path = matches.value_of("config").unwrap();
                match read_config_from_file(Path::new(&path)) {
                    Ok(config) => {
                        println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
                    },
                    Err(e) => panic!("Error with config: {:?}", e)
                }
            }
        },
        Some("checksum") => {
            if let Some(ref matches) = matches.subcommand_matches("checksum") {
                let key = matches.value_of("key").unwrap();
                match matches.value_of("config") {
                    Some(path) => {
                        match read_config_from_file(Path::new(&path)) {
                            Ok(config) => {
                                println!("{:?}", check_key_checksum(&key, &config.num_bytes));
                            },
                            Err(e) => println!("Error with config: {:?}", e)
                        }
                    },
                    None => {
                        let length = matches.value_of("length").expect("length required");
                        let len: i8 = length.parse().expect("Length must be a number");
                        println!("{:?}", check_key_checksum(&key, &len));
                    }
                };
            }
        },
        Some("create") => {
            if let Some(ref matches) = matches.subcommand_matches("create") {
                let config = match matches.is_present("config") {
                    true => {
                        // read existing config
                        let path = matches.value_of("config").unwrap();
                        match read_config_from_file(Path::new(path)) {
                            Ok(config) => config,
                            Err(e) => panic!("Error reading config: {:?}", e)
                        }
                    },
                    false => {
                        // create new config
                        let gen_hash = create_hash();
                        let hash = matches.value_of("hash").unwrap_or(&gen_hash[..]);
                        let len = matches.value_of("length").unwrap_or("8");
                        let num_bytes: i8 = len.parse().expect("Max bytes needs to be i8");
                        let mut blacklist: Vec<String> = vec![];
                        if matches.is_present("blacklist") {
                            // convert Vec<str> to Vec<String>
                            blacklist = matches.values_of("blacklist").unwrap().map(|s| s.to_string()).collect();
                        }
                        Config::new(hash, num_bytes, blacklist)
                    }
                };

                let input = matches.value_of("seed").unwrap();
                let seed = crc32::checksum_ieee(format!("{}+{}", input, &config.hash).as_bytes()) as i64;
                let key = make_key(&seed, &config.num_bytes, &config.byte_shifts);

                let config_json = config.config_to_json().expect("Error creating config");

                match matches.value_of("output") {
                    Some(path) => {
                        match write_config(&config_json, Path::new(path)) {
                            Err(e) => println!("Error saving config: {:?}", e),
                            Ok(_) => println!("Config file saved at {}", path)
                        }
                    },
                    None => {
                        if matches.is_present("json") {
                            let output_json = Output { key: key.clone(), config };
                            println!("{}", json!(output_json));
                        }

                        if !matches.is_present("config") && !matches.is_present("json") {
                            println!("Save this configuration data! You will need it to validate keys.");
                            println!("{}", config_json);
                            println!("--");
                            println!("{}", key);
                        }
                    }
                }

                // println!("{}", key);
                // println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
                // println!("{:?}", check_key_checksum(&key, &config.num_bytes));
            }
        },
        Some(_) => println!("Unrecognised command. Run keygen -h to see commands."),
        None => println!("No command. Run keygen -h to see commands.")
    }
}