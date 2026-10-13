use crate::{Error, Result};
use bip39::{Language, Mnemonic};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;

pub fn generate(words: u8, prefix: &str, raw_bits: Option<&str>) -> Result<Zeroizing<String>> {
    let bit_count = match words {
        12 => 128,
        24 => 256,
        _ => return Err(Error::Invalid("choose 12 or 24 words")),
    };
    let mut entropy = Zeroizing::new(vec![0u8; bit_count / 8]);
    if let Some(bits) = raw_bits {
        if !prefix.trim().is_empty()
            || bits.len() != bit_count
            || !bits.bytes().all(|b| b == b'0' || b == b'1')
        {
            return Err(Error::Invalid(
                "raw entropy must be exactly 128 or 256 binary digits, with no prefix",
            ));
        }
        for (i, b) in bits.bytes().enumerate() {
            entropy[i / 8] |= (b - b'0') << (7 - (i % 8));
        }
    } else {
        OsRng
            .try_fill_bytes(&mut entropy)
            .map_err(|_| Error::Invalid("OS randomness unavailable"))?;
        let prefix: Vec<&str> = prefix.split_whitespace().collect();
        if prefix.len() > bit_count / 11 {
            return Err(Error::Invalid("too many manual prefix words"));
        }
        for (i, word) in prefix.iter().enumerate() {
            let index = Language::English.find_word(word).ok_or(Error::Invalid(
                "manual prefix contains an unknown BIP39 word",
            ))? as usize;
            for bit in 0..11 {
                let position = i * 11 + bit;
                let mask = 1u8 << (7 - position % 8);
                entropy[position / 8] = (entropy[position / 8] & !mask)
                    | ((((index >> (10 - bit)) & 1) as u8) << (7 - position % 8));
            }
        }
    }
    let mnemonic =
        Mnemonic::from_entropy(&entropy).map_err(|_| Error::Invalid("entropy length"))?;
    Ok(Zeroizing::new(mnemonic.to_string()))
}
pub fn validate(phrase: &str) -> Result<Zeroizing<String>> {
    if phrase.len() > 1024 {
        return Err(Error::Invalid("phrase too long"));
    }
    let mnemonic = Mnemonic::parse_in(Language::English, phrase)
        .map_err(|_| Error::Invalid("BIP39 words or checksum are invalid"))?;
    if !matches!(mnemonic.word_count(), 12 | 24) {
        return Err(Error::Unsupported(
            "only 12 and 24 word phrases are enabled",
        ));
    }
    Ok(Zeroizing::new(mnemonic.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn official_zero_entropy_vectors() {
        assert_eq!(&*generate(12, "", Some(&"0".repeat(128))).unwrap(), "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about");
        assert_eq!(
            generate(24, "", Some(&"0".repeat(256)))
                .unwrap()
                .split_whitespace()
                .last(),
            Some("art")
        );
    }
    #[test]
    fn every_prefix_count_preserves_leading_zero_bits() {
        for words in [12, 24] {
            for count in 0..words as usize {
                let prefix = (0..count)
                    .map(|i| if i % 2 == 0 { "abandon" } else { "zoo" })
                    .collect::<Vec<_>>()
                    .join(" ");
                let phrase = generate(words, &prefix, None).unwrap();
                assert!(phrase.starts_with(&prefix));
                validate(&phrase).unwrap();
            }
        }
    }
    #[test]
    fn rejects_invalid_entropy() {
        assert!(generate(24, "notaword", None).is_err());
        assert!(generate(24, "", Some(&"0".repeat(255))).is_err());
        assert!(validate(&"abandon ".repeat(12)).is_err());
    }
}
