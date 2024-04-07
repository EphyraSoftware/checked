use anyhow::Context;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn open_file<P: AsRef<Path>>(path: P) -> anyhow::Result<BufWriter<File>> {
    let mut open_options = File::options();
    let open = open_options.create_new(true).write(true);
    #[cfg(target_family = "unix")]
    {
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

pub(crate) fn get_store_dir(input: Option<PathBuf>) -> anyhow::Result<PathBuf> {
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

pub(crate) fn get_signing_key_path<P: AsRef<Path>>(store_dir: P, name: &str) -> PathBuf {
    store_dir.as_ref().join(format!("{}.key", name))
}

pub(crate) fn get_verification_key_path<P: AsRef<Path>>(store_dir: P, name: &str) -> PathBuf {
    store_dir.as_ref().join(format!("{}.pub", name))
}

pub fn unix_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("system clock is incorrect");
    since_the_epoch.as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn open_a_file() {
        let f = tempfile::tempdir().unwrap();

        let mut r = open_file(f.path().join("test.txt")).unwrap();
        r.write("test".as_bytes()).unwrap();
        r.flush().unwrap();
    }
}
