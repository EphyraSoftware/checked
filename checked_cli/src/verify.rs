use crate::cli::VerifyArgs;
use minisign::{PublicKey, SignatureBox};
use std::io::BufReader;

/// Verification of an asset against a single signature.
///
/// Expects to find a file to verify and a signature to check against it.
/// Prints OK if the signature is valid and exits with an error otherwise.
pub fn verify(verify_args: VerifyArgs) -> anyhow::Result<()> {
    let vk = PublicKey::from_file(&verify_args.verification_key)?;

    let sig_path = verify_args.signature.clone().unwrap_or_else(|| {
        let p = verify_args.file.clone();
        p.with_extension(format!(
            "{}.minisig",
            verify_args
                .file
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        ))
    });

    let sig = SignatureBox::from_file(sig_path)?;

    let mut reader = BufReader::new(std::fs::File::open(&verify_args.file)?);

    match minisign::verify(&vk, &sig, &mut reader, false, false, false) {
        Ok(()) => {
            println!("Ok");
            Ok(())
        }
        Err(e) => {
            e.exit();
        }
    }
}
