extern crate rand;

// based on http://www.brandonstaggs.com/2007/07/26/implementing-a-partial-serial-number-verification-system-in-delphi/
use regex::Regex;
use self::rand::Rng;

#[derive(Debug, PartialEq)]
pub enum Status {
    Good,
    Invalid,
    Blacklisted,
    Phony
}

fn get_key_byte(seed: &i64, a: i16, b: i16, c: i16) -> String {
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

pub fn make_key(seed: &i64, num_bytes: &i8, byte_shifts: &Vec<(i16, i16, i16)>) -> String {
    let mut key_bytes: Vec<String> = vec![];

    for i in 0..*num_bytes {
        let index = i as usize;
        let shift = byte_shifts[index];
        key_bytes.push(get_key_byte(&seed, shift.0, shift.1, shift.2));
    }

    let mut result = format!("{:01$X}", seed, 8);
    for byte in key_bytes {
        result = format!("{}{}", result, byte);
    }

    result = format!("{}{}", result, get_checksum(&result[..]));

    // keys should always be 20 digits, but use chunks rather than loop to be sure
    let subs: Vec<&str> = result.split("").filter(|s| s.len() > 0).collect();
    let mut key: Vec<String> = vec![];
    for chunk in subs.chunks(4) {
        key.push(chunk.join(""));
    }

    key.join("-")
}

pub fn check_key_checksum(key: &str, num_bytes: &i8) -> bool {
    let s = key.replace("-", "").to_uppercase();
    let length = s.len();

    // length = 8 (seed) + 4 (checksum) + 2 * num_bytes
    if length != (12 + (2 * num_bytes)) as usize {
        return false;
    }

    let checksum = &s[length - 4..length];
    let slice = &s[..length - 4];

    checksum == get_checksum(&slice)
}

pub fn check_key(s: &str, blacklist: &Vec<String>, num_bytes: &i8, byte_shifts: &Vec<(i16, i16, i16)>) -> Status {
    if !check_key_checksum(s, num_bytes) {
        return Status::Invalid;
    }

    if *num_bytes < 3 {
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
    let mut bytes_to_check = 3;
    if *num_bytes > 5 {
        bytes_to_check = num_bytes / 2;
    }

    // pick random unchecked entry in bytes array and check it
    let mut checked: Vec<i8> = Vec::new();

    for _ in 0..bytes_to_check {
        let mut byte_to_check = rand::thread_rng().gen_range(0, num_bytes - 1);
        while checked.contains(&byte_to_check) {
            byte_to_check = rand::thread_rng().gen_range(0, num_bytes - 1);
        }
        checked.push(byte_to_check);

        let start = ((byte_to_check * 2) + 8) as usize;
        let end = start + 2;

        if end > key.len() {
            return Status::Invalid;
        }

        let key_byte = &key[start..end];
        let shifts = &byte_shifts[byte_to_check as usize];

        let byte = get_key_byte(&seed_num, shifts.0, shifts.1, shifts.2);
        if key_byte != byte {
            return Status::Phony;
        }
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
        let num_bytes = 4;
        let byte_shifts = vec![(24, 3, 200), (10, 0, 56), (1, 2, 91), (7, 1, 100)];
        assert_eq!("3ABC-9099-E39D-4E65-E060", super::make_key(&seed, &num_bytes, &byte_shifts));
    }

    #[test]
    fn test_check_key() {
        let key = "3ABC-9099-E39D-4E65-E060";
        let blacklist = vec![];
        let num_bytes = 4;
        let byte_shifts = vec![(24, 3, 200), (10, 0, 56), (1, 2, 91), (7, 1, 100)];
        assert_eq!(super::check_key(&key, &blacklist, &num_bytes, &byte_shifts), super::Status::Good);

        let inconsistent_key = "3abC-9099-e39D-4E65-E060";
        assert_eq!(super::check_key(&inconsistent_key, &blacklist, &num_bytes, &byte_shifts), super::Status::Good);

        let wrong_checksum = "3ABC-9099-E39D-4E65-E061";
        assert_eq!(super::check_key(&wrong_checksum, &blacklist, &num_bytes, &byte_shifts), super::Status::Invalid);

        let second_fake_key = "3ABC-9099-E49D-4E65-E761";
        assert_eq!(super::check_key(&second_fake_key, &blacklist, &num_bytes, &byte_shifts), super::Status::Phony);

        let third_fake_key = "3ABC-9199-E39D-4E65-EB61";
        assert_eq!(super::check_key(&third_fake_key, &blacklist, &num_bytes, &byte_shifts), super::Status::Phony);

        let invalid_key = "BC-9099-E39D-4E65-E061";
        assert_eq!(super::check_key(&invalid_key, &blacklist, &num_bytes, &byte_shifts), super::Status::Invalid);

        let second_invalid_key = "3AXC-9099-E39D-4E65-E061";
        assert_eq!(super::check_key(&second_invalid_key, &blacklist, &num_bytes, &byte_shifts), super::Status::Invalid);
    }

    #[test]
    fn test_blacklist() {
        let key = "3ABC-9099-E39D-4E65-E060";
        let blacklist = vec!["3abc9099"];
        let num_bytes = 4;
        let byte_shifts = vec![(24, 3, 200), (10, 0, 56), (1, 2, 91), (7, 1, 100)];

        assert_eq!(super::check_key(&key, &blacklist, &num_bytes, &byte_shifts), super::Status::Blacklisted);
    }
}
