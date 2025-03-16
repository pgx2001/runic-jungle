mod decimal;
pub mod etch;
mod inscription;
pub mod transfer;

use std::str::FromStr;

use crate::bitcoin_lib::Amount;
use ordinals::{Etching, SpacedRune};

const DEFAULT_POSTAGE: u64 = 10_000;
const TARGET_POSTAGE: Amount = Amount::from_sat(10_000);
const MAX_STANDARD_OP_RETURN_SIZE: usize = 83;

pub fn validate_etching(
    runename: &str,
    symbol: Option<u32>,
    divisibility: u8,
    total_supply: u128,
) -> Result<(SpacedRune, u128, Option<char>), String> {
    let spaced_rune = match SpacedRune::from_str(runename) {
        Err(_) => return Err("Failed to convert into Spaced Rune".to_string()),
        Ok(sr) => sr,
    };

    if spaced_rune.rune.is_reserved() {
        return Err(format!("rune `{}` is reserved", spaced_rune.rune));
    }

    if divisibility > Etching::MAX_DIVISIBILITY {
        return Err(String::from(
            "DIVISIBILITY must be less than or equal to 38",
        ));
    }

    let total_supply = total_supply * 10u128.pow(divisibility as u32);

    if total_supply == 0 {
        return Err(String::from("Supply must be over 0"));
    }

    let symbol = match symbol {
        None => None,
        Some(codepoint) => {
            let symbol = match char::from_u32(codepoint) {
                None => return Err(String::from("Not a valid unicode for symbol")),
                Some(unicode) => unicode,
            };
            Some(symbol)
        }
    };
    Ok((spaced_rune, total_supply, symbol))
}
