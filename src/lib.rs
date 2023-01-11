use std::string::FromUtf8Error;

/// Data structure for fast Playfair encoding and decoding of text.
///
/// # Example
///
/// ```
/// use playfair::PlayfairCipher;
/// let cipher = PlayfairCipher::new("Hello Playfair Cipher");
/// let a = "This is a test.";
/// let b = cipher.encode(a).unwrap();
/// let c = cipher.decode(&b).unwrap();
/// assert_eq!(a, c);
/// ```
pub struct PlayfairCipher {
    // Maps a letter index (0 to 24 inclusively) to a position which is encoded
    // as row*8 + col, where row and col are numbers from 1 to 5 inclusively.
    positions: [u8; 25],
    // Maps a position encoded as row*8 + col, where row and col are numbers
    // from 0 to 6 inclusively to the respective letter ('a' to 'z').
    // If an index falls outside the range from 1 to 5 inclusively, then 0 will
    // be mapped to 5 and 6 to 1 (wrap around).
    letters: [u8; 64],
}

impl PlayfairCipher {
    const X_INDEX: u8 = b'x' - b'a' - 1;
    const IJ_INDEX: u8 = b'i' - b'a';

    pub fn print(&self) {
        let mut s = String::new();
        for row in 1..=5 {
            for col in 1..=5 {
                s.push(self.letters[col + 8 * row] as char);
            }
            s.push('\n');
        }
        println!("{}", s);
    }

    pub fn new(key: &str) -> Self {
        let mut positions = [255u8; 25];
        let mut letters = [0u8; 64];
        let mut pos = (1, 1);
        // fill square with characters from `key`
        for &letter in key.as_bytes() {
            let letter_index = if letter < b'j' {
                letter.wrapping_sub(b'a')
            } else {
                letter.wrapping_sub(b'a' + 1)
            } as usize;
            if letter_index >= 25 {
                // ignore characters which are non-alphabetical or non-lowercase
                continue;
            }
            if positions[letter_index] != 255u8 {
                // Already taken?
                continue;
            }

            // update `positions` and `letters`
            let encoded_pos = pos.0 * 8 + pos.1;
            positions[letter_index] = encoded_pos;
            letters[encoded_pos as usize] = if letter == b'j' { b'i' } else { letter };

            // Go to next valid `pos`
            pos.1 += 1;
            if pos.1 == 6 {
                pos = (pos.0 + 1, 1);
            }
        }

        // fill the rest of the square with the remaining lower-case characters in alphabetical order
        for (letter_index, position) in positions.iter_mut().enumerate() {
            if *position != 255 {
                continue;
            }

            // update `position` and `letters`
            let encoded_pos = pos.0 * 8 + pos.1;
            *position = encoded_pos;
            letters[encoded_pos as usize] = if (letter_index as u8) <= Self::IJ_INDEX {
                letter_index as u8 + b'a'
            } else {
                letter_index as u8 + b'a' + 1
            };

            // go to next valid pos
            pos.1 += 1;
            if pos.1 == 6 {
                pos = (pos.0 + 1, 1);
            }
            debug_assert!(pos <= (6, 1));
        }

        // Set 8-neighbors of the square with wrap-around values
        for row in 1..=5 {
            letters[row * 8] = letters[row * 8 + 5];
            letters[row * 8 + 6] = letters[row * 8 + 1];
        }
        for col in 0..=6 {
            letters[col] = letters[col + 5 * 8];
            letters[col + 6 * 8] = letters[col + 8];
        }

        Self { positions, letters }
    }

    pub fn encode(&self, text: &str) -> Result<String, FromUtf8Error> {
        self.encode_or_decode(text, true)
    }

    pub fn decode(&self, text: &str) -> Result<String, FromUtf8Error> {
        self.encode_or_decode(text, false)
    }

    pub fn encode_or_decode(&self, text: &str, is_encode: bool) -> Result<String, FromUtf8Error> {
        let mut result = Vec::<u8>::with_capacity(text.len() + 1);
        let mut last_pos = None;
        for &c in text.as_bytes() {
            let letter_index = c.wrapping_sub(b'a');
            if letter_index >= 26 {
                result.push(c);
                continue;
            }
            let letter_index = if letter_index <= Self::IJ_INDEX {
                letter_index
            } else {
                letter_index - 1
            };
            if let Some(pos) = last_pos {
                if result[pos] == letter_index {
                    // insert an 'x' to split double letter
                    let (a, b) = self.encode_or_decode_pair(result[pos], Self::X_INDEX, is_encode);
                    result[pos] = a;
                    result.push(b);
                } else {
                    let (a, b) = self.encode_or_decode_pair(result[pos], letter_index, is_encode);
                    last_pos = None;
                    result[pos] = a;
                    result.push(b);
                    continue;
                }
            }
            last_pos = Some(result.len());
            result.push(letter_index);
        }
        if let Some(pos) = last_pos {
            let (a, b) = self.encode_or_decode_pair(result[pos], Self::X_INDEX, is_encode);
            result[pos] = a;
            result.push(b);
        }

        String::from_utf8(result)
    }

    fn encode_or_decode_pair(&self, a: u8, b: u8, is_encode: bool) -> (u8, u8) {
        let pos_a = self.positions[a as usize];
        let pos_b = self.positions[b as usize];
        if pos_a == pos_b {
            if a == Self::X_INDEX {
                // Case not really defined in the Playfair Cipher description of Wikipedia. Let's improvise.
                (a, b)
            } else {
                self.encode_or_decode_pair(a, Self::X_INDEX, is_encode)
            }
        } else if (pos_a & 7) == (pos_b & 7) {
            // same column
            if is_encode {
                (
                    self.letters[(pos_a + 8) as usize],
                    self.letters[(pos_b + 8) as usize],
                )
            } else {
                (
                    self.letters[(pos_a - 8) as usize],
                    self.letters[(pos_b - 8) as usize],
                )
            }
        } else if (pos_a & 0o70) == (pos_b & 0o70) {
            // same row
            if is_encode {
                (
                    self.letters[(pos_a + 1) as usize],
                    self.letters[(pos_b + 1) as usize],
                )
            } else {
                (
                    self.letters[(pos_a - 1) as usize],
                    self.letters[(pos_b - 1) as usize],
                )
            }
        } else {
            let pos_c = (pos_a & 0o70) | (pos_b & 7);
            let pos_d = (pos_b & 0o70) | (pos_a & 7);
            (self.letters[pos_c as usize], self.letters[pos_d as usize])
        }
    }
}

#[test]
fn test_playfair_cipher_simple_test() {
    let cipher = PlayfairCipher::new("Hello Playfair Cipher");
    let a = "This is a test.";
    let b = cipher.encode(a).unwrap();
    let c = cipher.decode(&b).unwrap();
    assert_eq!(a, c);
}

#[test]
fn test_playfair_cipher_wikipedia_example() {
    let cipher = PlayfairCipher::new("playfair example");
    let a = "hide the gold in the tree stump";
    let b = cipher.encode(a).unwrap();
    assert_eq!(b, "bmod zbx dnab ek udm uixmm ouvif");
}

#[test]
fn test_playfair_cipher_attack_at_dawn() {
    let cipher = PlayfairCipher::new("gravity falls");
    let a = "attack at dawn";
    let b = cipher.encode(a).unwrap();
    assert_eq!(b, "gffgbm gf nfaw");
}
