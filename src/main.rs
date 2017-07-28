extern crate regex;
extern crate crc;

mod keygen;

use keygen::{ make_key, check_key };
use crc::{ crc32 };

fn main() {
	// black list is array of seeds
	let blacklist = vec!["11111111"];

	let seed = crc32::checksum_ieee(format!("{}+{}", "jamestease@gmail.com", "hello").as_bytes()) as i64;
	println!("{:?}", seed);
    // let seed = 0x3abc9099; // jamestease@gmail.com+hello crc32 http://www.fileformat.info/tool/hash.htm
    // let key = "3ABC-9099-E39D-4E65-E060";

    // TODO: crc32
    // TODO: create config and give to user eg bytes and transforms used, hash for crc32
    // TODO: flow: keygen new myname@example.com => 1234-1234-1234-1234
    // keygen verify 1234-1324-1234-1234 => Status
    let key = make_key(&seed);
    println!("{}", key);
    println!("{:?}", check_key(&key, &blacklist));
}