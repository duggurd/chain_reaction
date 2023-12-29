use std::time::{SystemTime, UNIX_EPOCH};
use rand;

pub(crate) fn generate_unique_id() -> u64 {
    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64;

    // Get current process ID
    let process_id = std::process::id() as u64;

    // Generate a random number
    let random_number: u64 = rand::random();

    // Combine the components to create a unique ID
    let unique_id = timestamp ^ process_id ^ random_number;

    unique_id
}