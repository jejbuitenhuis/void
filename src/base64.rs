pub struct Base64 {}

impl Base64 {
	const ALPHABET: &'static [u8] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_".as_bytes();

	fn index_of_char(char: char) -> Option<u64> {
		Base64::ALPHABET.iter()
			.position(|&c| c == char as u8)
			.map(|i| i as u64)
	}

	pub fn encode(num: u64) -> String {
		// num == 0 doesn't work because of the `while num > 0`
		if num == 0 {
			return ( Base64::ALPHABET[0] as char ).to_string();
		}

		let mut num = num;
		let mut result = String::new();
		let mut bit_parts: Vec<u8> = Vec::new();

		while num > 0 {
			// get the trailing 6 bits
			// force the cast from u64 to u8 because we know it will fit
			let bit_part: u8 = (num & 0b111111).try_into()
				.expect("Error casting u64 to u8 in Base64::encode");

			bit_parts.push(bit_part);

			// remove the trailing 6 bits
			num = num >> 6;
		}

		for part in bit_parts.iter() {
			// get the character corresponding with the 6 bits (the index in
			// the alphabet)
			result.push( Base64::ALPHABET[*part as usize] as char );
		}

		result
	}

	pub fn decode(string: String) -> Result<u64, String> {
		let mut result = 0u64;

		// reverse of `Base64::encode`
		for char in string.chars().rev() {
			let index = match Base64::index_of_char(char) {
				Some(c) => c,
				None => return Err(
					format!("Invalid character found in supplied string ('{}')", char)
				),
			};

			result = (result << 6) + index;
		}

		Ok(result)
	}
}
