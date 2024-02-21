use chrono::{DateTime, Utc};
use hdi::prelude::*;
use pgp::types::KeyTrait;
use pgp::{
    armor::{BlockType, Dearmor},
    from_bytes_many, PublicOrSecret, SignedPublicKey,
};
use std::io::Cursor;
use std::{
    collections::BTreeMap,
    io::{Read, Seek},
};

pub fn try_extract_public_key(input: String) -> ExternResult<SignedPublicKey> {
    let c = Cursor::new(input.as_bytes());
    let (typ, _headers, bytes) = parse(c).unwrap();

    if typ != Some(BlockType::PublicKey) {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Expected a public key".to_string()
        )));
    }

    let content = from_bytes_many(&bytes[..])
        .collect::<Result<Vec<PublicOrSecret>, _>>()
        .map_err(|e| {
            wasm_error!(WasmErrorInner::Guest(format!(
                "Error parsing PGP data: {}",
                e
            )))
        })?;

    if content.len() != 1 {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Expected exactly one PGP key".to_string()
        )));
    }

    match &content[0] {
        PublicOrSecret::Public(public_key) => Ok(public_key.clone()),
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "Expected a public key".to_string()
        ))),
    }
}

pub struct PublicKeySummary {
    pub fingerprint: String,
    pub name: String,
    pub email: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl PublicKeySummary {
    pub fn try_from_public_key(public_key: &SignedPublicKey) -> ExternResult<Self> {
        let fingerprint = public_key.fingerprint();

        match public_key.details.users.first() {
            None => Err(wasm_error!(WasmErrorInner::Guest(
                "No user id found on the public key".to_string()
            ))),
            Some(user_id) => {
                let user_id = user_id.id.id().to_string();

                let pattern = regex::Regex::new(r"^([^ ]+)( <([^>]+)>)?$").unwrap();
                let captures = pattern.captures(&user_id).ok_or_else(|| {
                    wasm_error!(WasmErrorInner::Guest("Invalid user id format".to_string()))
                })?;

                let name = match captures.get(1) {
                    Some(name) => name.as_str().to_string(),
                    None => {
                        return Err(wasm_error!(WasmErrorInner::Guest(
                            "Missing user name".to_string()
                        )));
                    }
                };
                let email = captures.get(3).map(|email| email.as_str().to_string());

                Ok(PublicKeySummary {
                    fingerprint: hex::encode(fingerprint).to_uppercase(),
                    name,
                    email,
                    expires_at: public_key.expires_at(),
                })
            }
        }
    }
}

type PubicKeyParsed = (Option<BlockType>, BTreeMap<String, String>, Vec<u8>);

fn parse<R: Read + Seek>(mut input: R) -> std::io::Result<PubicKeyParsed> {
    let mut dearmor = Dearmor::new(input.by_ref());

    let mut bytes = Vec::new();
    dearmor.read_to_end(&mut bytes)?;

    Ok((dearmor.typ, dearmor.headers, bytes))
}

#[cfg(test)]
mod tests {
    use pgp::{types::KeyTrait, PublicKey};

    use super::*;

    const TEST_KEY_1: &str = r#"
-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEZc/RVBYJKwYBBAHaRw8BAQdA/pL2Vk0GwBWhsMRgpoeI2Os0UlxpWFHzdvuH
XUHKnjG0BnRlc3RlcoiZBBMWCgBBFiEE+EdcI9xicATMCx3Benquor8GXMoFAmXP
0VQCGwMFCQWjmoAFCwkIBwICIgIGFQoJCAsCBBYCAwECHgcCF4AACgkQenquor8G
XMqa9gD/Vtvj91GMGfYujOXVOO9r1EEaz/J/lKwOGemVJPQG0UQBAI6tDkiCEruX
7ra/pO5frPMqUjJeqNhUgH7bQHwqZz4LuDgEZc/RVBIKKwYBBAGXVQEFAQEHQAI/
BVg8bP9fKzKRO1uvGm6cpo8Ht+X9JkSnsMacnRUNAwEIB4h4BBgWCgAgFiEE+Edc
I9xicATMCx3Benquor8GXMoFAmXP0VQCGwwACgkQenquor8GXMogCQD+KNg9+ukK
oSr87qDC0dehXKeYN7NiR985Y5TtvRTkZeoA/ROsHS3dzWIMjHOn8kWNU8n93VUt
x//CtSMXRrqvlAIF
=zuwf
-----END PGP PUBLIC KEY BLOCK-----
    "#;

    const TEST_KEY_2: &str = r#"
-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEZc/svhYJKwYBBAHaRw8BAQdA0eZIgnzMYpUS07cPQxkHob6T53nt8o3qw58t
JdFTxmC0H3Rlc3RlcjIgPHRlc3RlcjJAbWFpbGhhcHB5LmNvbT6ImQQTFgoAQRYh
BFl3mfcvqmAioFNJofiLypseg5Z4BQJlz+y+AhsDBQkFo5qABQsJCAcCAiICBhUK
CQgLAgQWAgMBAh4HAheAAAoJEPiLypseg5Z4qsgBAL/hkFGa+tJvdqxs639b/WhJ
N6ouTIcLqaCKIjdvEy/kAQCLrKOtLKNshnTM8whhMr4Akf4OCSRS0sfZqBeFSSqZ
Cbg4BGXP7L4SCisGAQQBl1UBBQEBB0A6enyZUTt8Leo7ZFlG68L6b+kL5VLsQjpe
AYs3iJnSSAMBCAeIfgQYFgoAJhYhBFl3mfcvqmAioFNJofiLypseg5Z4BQJlz+y+
AhsMBQkFo5qAAAoJEPiLypseg5Z4rRAA/A4sNX8GyWopybZJGXu++lJ4lY1/Rgj8
oo0Dy+qQjDoTAPwMQD9dqZJX0eDz2j9JhyOiVI7ezjRTTef0+r4Ox152Bg==
=Arm4
-----END PGP PUBLIC KEY BLOCK-----
    "#;

    #[test]
    fn extract() {
        let public_key = try_extract_public_key(TEST_KEY_2.to_string()).unwrap();

        assert_eq!(
            "tester2 <tester2@mailhappy.com>",
            public_key.details.users.first().unwrap().id.id()
        );
        assert_eq!(
            vec![
                89, 119, 153, 247, 47, 170, 96, 34, 160, 83, 73, 161, 248, 139, 202, 155, 30, 131,
                150, 120
            ],
            public_key.fingerprint()
        );
    }

    #[test]
    fn summary() {
        let public_key = try_extract_public_key(TEST_KEY_2.to_string()).unwrap();

        let summary = PublicKeySummary::try_from_public_key(&public_key).unwrap();

        assert_eq!(
            "597799F72FAA6022A05349A1F88BCA9B1E839678",
            summary.fingerprint
        );
        assert_eq!("tester2", summary.name);
        assert_eq!(Some("tester2@mailhappy.com".to_string()), summary.email);
    }

    #[test]
    fn summary_no_email() {
        let public_key = try_extract_public_key(TEST_KEY_1.to_string()).unwrap();

        let summary = PublicKeySummary::try_from_public_key(&public_key).unwrap();

        assert_eq!(
            "F8475C23DC627004CC0B1DC17A7AAEA2BF065CCA",
            summary.fingerprint
        );
        assert_eq!("tester", summary.name);
        assert_eq!(None, summary.email);
    }
}
