# Keygen

## Installation

Windows/Linux: Compile from source using Rust - no binaries at the mo.

Mac: target/release/keygen

## Usage

Creates a serial key from a seed, based [on this article](http://www.brandonstaggs.com/2007/07/26/implementing-a-partial-serial-number-verification-system-in-delphi/).

The serial key can be checked for validity, either by using a checksum or by verifying bytes in the key.


### `keygen create`
Creates a key from an initial string value.

`-s --seed REQUIRED`
Initial value to use to create the string

```keygen create -s myname@example.com```

`-h --hash`
String to combine with the seed when creating the key. If you don't provide a hash, a random one will be generated and used.

```keygen create -s myname@example.com -h aLongstringValueasaHash```

`-c -- config`
Path to existing JSON config file. This file will be read when creating the key.

```keygen create -s myname@example.com -c config.json```

`-b -- blacklist MULTIPLE`
Seed values to add to a blacklist. Any keys with seed values in the blacklist will fail verification.

```keygen create -s myname@examplecom -b 11111111 -b 22222222```

`-l --length`
Number of bytes to create in the key. Longer keys should be tricker to crack but are more difficult for users to type. Keys are made up of the seed (8 characters long), a length of bytes (2 characters per byte), then the checksum (4 characters).

```keygen create -s myname@example.com -l 12 // => 53A5-4C20-7474-CC94-AF39-B63D-8B34-54A7-C1DA
keygen create -s myname@example.com -l 4 // => 53A5-4C20-A43E-6490-052A```

`-o --output`
Path to which to write config settings. When creating a key it's vital to save the config used to create the key, otherwise you won't be able to verify it. If you provide an output value, then a file with the value's name will be created and the config will be saved there as JSON. If this value isn't provided, JSON config will be printed out to the terminal.

`keygen create -s myname@example.com -o config.json`

### `keygen verify`
Check that a key is valid. Returns `Keygen::Status`.

```keygen verify -k 1234-5678-abcd-1234 -c config.json // => Status::Good, Status::Invalid, Status::Blacklisted, Status::Phony```

`-k --key REQUIRED`
Key to check.

`-c --config REQUIRED`
Path to JSON config file; must be the config used to create the key.

### `keygen checksum`
Check that a key's checksum is correct (ie that the key hasn't been altered). Returns `bool`

```keygen verify -k 1234-5678-abcd-1234 -c config.json // => true```

`-k --key REQUIRED`
Key to check.

`-c --config REQUIRED`
Path to JSON config file; must be the config used to create the key.


## Seeds

Seeds are created from the input and hash being run through `CRC32` - they are always eight characters long in hexadecimal format.

## Config

The config is vital once a key has been created - if you don't save the config, or try to use a different one to verify a key, you won't be able to validate keys! The config file contains information which can be used to generate keys, so needs to be kept secret. You can use the same config to generate multiple keys by using the `-c` argument when creating a key.

### Example configuration
```{
    "num_bytes": 8,             // length of key
    "byte_shifts": [            // array of arrays - these are used to check each byte
        [62, 252, 46],
        [57, 195, 131],
        [21, 251, 32],
        [129, 94, 254],
        [205, 24, 45],
        [161, 36, 17],
        [88, 109, 26],
        [105, 237, 248]
    ],
    "hash": "hash",             // the hash used in creating the string
    "blacklist": ["11111111"]   // blacklisted seeds
}```

## Verification/Checksum

The `keygen checksum` method is a quick and fairly inaccurate way to determine whether a key is invalid or not - it only checks whether the checksum matches the rest of the key, although it is possible to alter the key and checksum and still end up valid. The verify function checks the checksum along with several bytes in the key to determine if the key actually is valid - the full reasoning is [in the original article](http://www.brandonstaggs.com/2007/07/26/implementing-a-partial-serial-number-verification-system-in-delphi/), but essentially this is so you have a quick-and-dirty way to check the key's valid on startup (`checksum`), and a full check later when trying to use more funcationlity like saving. Having the quick check on startup means reverse-engineers don't know exactly when the full check happens in the software so there isn't an obvious entry point.


