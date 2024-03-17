use crate::cli::GenerateArgs;
use anyhow::Context;
use minisign::KeyPair;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

fn open_file<P: AsRef<Path>>(path: P) -> anyhow::Result<BufWriter<File>> {
    let mut open_options = File::options();
    let open = open_options.create_new(true).write(true);
    if cfg!(unix) {
        use std::os::unix::prelude::OpenOptionsExt;
        open.mode(0o644);
    }

    match open.open(path.as_ref()) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(anyhow::anyhow!("File already exists - {:?}", path.as_ref()))
        }
        r => r
            .map(BufWriter::new)
            .with_context(|| format!("Could not open - {:?}", path.as_ref())),
    }
}

fn get_store_dir(input: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let dir = match input {
        Some(p) => Ok(p),
        None => if cfg!(unix) || cfg!(windows) {
            dirs::home_dir().map(|p| p.join(".checked"))
        } else {
            None
        }
        .ok_or_else(|| anyhow::anyhow!("Could not determine the store directory")),
    }?;

    if !dir.exists() {
        std::fs::create_dir(&dir)
            .map_err(|e| anyhow::anyhow!("Could not create store directory: {}", e))?;
    }

    Ok(dir)
}

pub fn generate(generate_args: GenerateArgs) -> anyhow::Result<()> {
    let store_dir = get_store_dir(generate_args.path)?;

    // Signing key
    let sk_path = store_dir.join(format!("{}.key", generate_args.name));
    let mut sk_file = open_file(&sk_path)?;

    // Verification key
    let vk_path = store_dir.join(format!("{}.pub", generate_args.name));
    let mut vk_file = open_file(&vk_path)?;

    #[cfg(not(any(windows, unix)))]
    let password = generate_args.password;
    #[cfg(any(windows, unix))]
    let password = rpassword::prompt_password("New password: ")?;

    let _pk = KeyPair::generate_and_write_encrypted_keypair(
        &mut vk_file,
        &mut sk_file,
        None,
        Some(password),
    )?
    .pk;

    sk_file.flush()?;
    vk_file.flush()?;

    println!(
        "\nThe secret key was saved as {} - Keep it secret!",
        sk_path.display()
    );
    println!(
        "The public key was saved as {} - That one can be public.\n",
        vk_path.display()
    );
    // println!("Files signed using this key can be verified with the following command:\n");
    // println!("checked verify <file> -P {}", _pk.to_base64());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_a_file() {
        let f = tempfile::tempdir().unwrap();

        let mut r = open_file(f.path().join("test.txt")).unwrap();
        r.write("test".as_bytes()).unwrap();
        r.flush().unwrap();
    }
}
