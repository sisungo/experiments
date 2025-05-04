use rand::Rng;
use std::{sync::atomic::{AtomicU16, Ordering}, time::SystemTime};

/// Generates a file name that is random enough.
pub fn filename(prefix: &str, extension: &str) -> String {
    const VALID_CHARACTERS: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const RANDOM_LEN: usize = 32;

    static PERIOD: AtomicU16 = AtomicU16::new(0);

    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    let mut random_part = String::with_capacity(RANDOM_LEN);
    let period = PERIOD.fetch_add(1, Ordering::Relaxed);

    for _ in 0..RANDOM_LEN {
        random_part
            .push(VALID_CHARACTERS[rand::rng().random_range(0..VALID_CHARACTERS.len())] as char);
    }

    format!("{prefix}-{timestamp}-{period}-{random_part}.{extension}")
}
