extern crate regex;
extern crate crc;
extern crate clap;

mod keygen;

use clap::{ App, Arg, SubCommand };
use keygen::{ make_key, check_key, check_key_checksum };
use crc::{ crc32 };

#[derive(Debug)]
pub struct Config<'a> {
    num_bytes: i8, // TODO: min num
    byte_shifts: Vec<(i16, i16, i16)>, // TODO: must be 3 long
    hash: &'a str,
    blacklist: Vec<&'a str>
}

fn main() {
    let config = Config {
        num_bytes: 4,
        byte_shifts: vec![(24, 3, 200), (10, 0, 56), (1, 2, 91), (7, 1, 100)],
        hash: "hello",
        blacklist: vec!["11111111"]
    };

    println!("{:?}", config);

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
                         .help("String used to create a key"))
                    .subcommand(SubCommand::with_name("verify")
                                .about("Check if key is valid")
                                .arg(Arg::with_name("KEY")
                                        .required(true)))
                    .subcommand(SubCommand::with_name("checksum")
                                .about("Check if key checksum is valid")
                                .arg(Arg::with_name("KEY")
                                     .required(true)))
                    .get_matches();

    let verify = matches.subcommand_matches("verify");
    match verify {
        Some(arg) => {
            let key = arg.value_of("KEY").unwrap(); // required so unwrap ok
            println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));
        },
        None => {}
    }

    let checksum = matches.subcommand_matches("checksum");
    match checksum {
        Some(arg) => {
            let key = arg.value_of("KEY").unwrap();
            println!("{:?}", check_key_checksum(&key));
        },
        None => {}
    }

    let create = matches.value_of("SEED");
    match create {
        Some(input) => {
            let seed = crc32::checksum_ieee(format!("{}+{}", input, &config.hash).as_bytes()) as i64;
            let key = make_key(&seed, &config.num_bytes, &config.byte_shifts);
            println!("{}", key);
            println!("{:?}", check_key(&key, &config.blacklist, &config.num_bytes, &config.byte_shifts));

        },
        None => {}
    };
}