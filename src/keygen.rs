// based on http://www.brandonstaggs.com/2007/07/26/implementing-a-partial-serial-number-verification-system-in-delphi/
use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum Status {
	Good,
	Invalid,
	Blacklisted,
	Phony
}

fn get_key_byte(seed: &i64, a: u32, b: u32, c: u32) -> String {
	let a_shift = a % 25;
	let b_shift = b % 3;
	let mut result;

	if a_shift % 2 == 0 {
		result = ((seed >> a_shift) & 0x000000FF) ^ ((seed >> b_shift) | c as i64);
	} else {
		result = ((seed >> a_shift) & 0x000000FF) ^ ((seed >> b_shift) & c as i64);
	}

	result = result & 0xFF;

	// formats to uppercase hex, must be 2 chars with leading 0s
	// https://doc.rust-lang.org/std/fmt/#width
	format!("{:01$X}", result, 2)
}

fn get_checksum(s: &str) -> String {
	let mut left = 0x0056; // 101
	let mut right: u16 = 0x00AF; // 175

	for ch in s.chars() {
		right += ch as u16;
		if right > 0x00FF {
			right -= 0x00FF;
		}

		left += right;
		if left > 0x00FF {
			left -= 0x00FF;
		}
	}

	let sum = (left << 8) + right;

	// format as upperhex with leading 0s, must be 4 chars long
	format!("{:01$X}", sum, 4)
}

pub fn make_key(seed: &i64) -> String {
	let mut key_bytes: Vec<String> = vec![];
	key_bytes.push(get_key_byte(&seed, 24, 3, 200));
	key_bytes.push(get_key_byte(&seed, 10, 0, 56));
	key_bytes.push(get_key_byte(&seed, 1, 2, 91));
	key_bytes.push(get_key_byte(&seed, 7, 1, 100));


	let mut result = format!("{:X}", seed);
	for byte in key_bytes {
		result = format!("{}{}", result, byte);
	}

	result = format!("{}{}", result, get_checksum(&result[..]));

	// let mut subs: Vec<&str> = vec![];
	// let mut i = 0;
	// let step = 4;

	// while i < result.len() {
	// 	let mut end = i + step;
	// 	if end > result.len() {
	// 		end = result.len();
	// 	}
	// 	subs.push(&result[i..end]);
	// 	i += step;
	// }
	// subs.join("-")

	// keys should always be 20 digits, but use chunks rather than loop to be sure
	let subs: Vec<&str> = result.split("").filter(|s| s.len() > 0).collect();
	let mut key: Vec<String> = vec![];
	for chunk in subs.chunks(4) {
		key.push(chunk.join(""));
	}

	key.join("-")
}

pub fn check_key_checksum(key: &str) -> bool {
	let mut result = false;
	let s = key.replace("-", "").to_uppercase();
	let length = s.len();

	if length != 20 {
		return result;
	}

	let checksum = &s[length - 4..length];
	let slice = &s[..16];
	result = checksum == get_checksum(&slice);
	result
}

pub fn check_key(s: &str, blacklist: &Vec<&str>) -> Status {
	if !check_key_checksum(s) {
		return Status::Invalid;
	}

	let key = s.replace("-", "").to_uppercase();
	let seed: String = key.chars().take(8).collect();

	for item in blacklist {
		if seed == item.to_uppercase() {
			return Status::Blacklisted;
		}
	}


	let re = Regex::new(r"[A-F0-9]{8}").unwrap();
	if !re.is_match(&seed) {
		return Status::Phony;
	}

	let seed_num = match i64::from_str_radix(&seed[..], 16) {
		Err(_) => return Status::Invalid,
		Ok(s) => s
	};

	// test key_bytes - don't test all of them!
	let key_byte = &key[8..10];

	// these need to be the same params used in key generation
	let byte = get_key_byte(&seed_num, 24, 3, 200);
	if key_byte != byte {
		return Status::Phony;
	}

	let second_key_byte = &key[10..12];
	let second_byte = get_key_byte(&seed_num, 10, 0, 56);
	if second_key_byte != second_byte {
		return Status::Phony;
	}

	let third_byte = &key[12..14];
	let third_key_byte = get_key_byte(&seed_num, 1, 2, 91);
	if third_key_byte != third_byte {
		return Status::Phony;
	}

	let fourth_byte = &key[14..16];
	let fourth_key_byte = get_key_byte(&seed_num, 7, 1, 100);
	if fourth_key_byte != fourth_byte {
		return Status::Phony;
	}

	return Status::Good;
}

#[cfg(test)]
mod test {
	#[test]
	fn test_bytes() {
		let seed = 0xA2791717;
		assert_eq!(super::get_key_byte(&seed, 24, 3, 200), "7D");
		assert_eq!(super::get_key_byte(&seed, 10, 0, 56), "7A");
		assert_eq!(super::get_key_byte(&seed, 1, 2, 91), "CA");
		assert_eq!(super::get_key_byte(&seed, 7, 1, 100), "2E");
	}

	#[test]
	fn test_get_checksum() {
		let key = "A279-1717-7D7A-CA2E-7154";
		assert_eq!(super::get_checksum(key), "49DA");

		let second_key = "3ABC-9099-E39D-4E65-E060";
		assert_eq!(super::get_checksum(second_key), "82F0");
	}

	#[test]
	fn test_make_key() {
		let seed = 0x3abc9099;
		assert_eq!("3ABC-9099-E39D-4E65-E060", super::make_key(&seed));
	}

	#[test]
	fn test_check_key() {
		let key = "3ABC-9099-E39D-4E65-E060";
		let blacklist = vec![];
		assert_eq!(super::check_key(&key, &blacklist), super::Status::Good);

		let inconsistent_key = "3abC-9099-e39D-4E65-E060";
		assert_eq!(super::check_key(&inconsistent_key, &blacklist), super::Status::Good);

		let wrong_checksum = "3ABC-9099-E39D-4E65-E061";
		assert_eq!(super::check_key(&wrong_checksum, &blacklist), super::Status::Invalid);

		let second_fake_key = "3ABC-9099-E49D-4E65-E761";
		assert_eq!(super::check_key(&second_fake_key, &blacklist), super::Status::Phony);

		let third_fake_key = "3ABC-9199-E39D-4E65-EB61";
		assert_eq!(super::check_key(&third_fake_key, &blacklist), super::Status::Phony);

		let invalid_key = "BC-9099-E39D-4E65-E061";
		assert_eq!(super::check_key(&invalid_key, &blacklist), super::Status::Invalid);

		let second_invalid_key = "3AXC-9099-E39D-4E65-E061";
		assert_eq!(super::check_key(&second_invalid_key, &blacklist), super::Status::Invalid);
	}

	#[test]
	fn test_blacklist() {
		let key = "3ABC-9099-E39D-4E65-E060";
		let blacklist = vec!["3abc9099"];

		assert_eq!(super::check_key(&key, &blacklist), super::Status::Blacklisted);
	}
}
