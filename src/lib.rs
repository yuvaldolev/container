use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

// use nix::sched;
use nix::sys::socket;
use nix::sys::socket::{AddressFamily, SockFlag, SockType};

mod opts;
pub use opts::Opts;

pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    // Choose the container's hostname.
    let hostname = choose_hostname();

    // Create a socketpair used to send messages from the parent to the child.
    let (parent_socket, child_socket) = socket::socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::empty(),
    )?;

    Ok(())
}

fn choose_hostname() -> String {
    const MAJOR: [&str; 22] = [
        "fool",
        "magician",
        "high-priestess",
        "empress",
        "emperor",
        "hierophant",
        "lovers",
        "chariot",
        "strength",
        "hermit",
        "wheel",
        "justice",
        "hanged-man",
        "death",
        "temperance",
        "devil",
        "tower",
        "star",
        "moon",
        "sun",
        "judgment",
        "world",
    ];
    const MINOR: [&str; 14] = [
        "ace", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "page",
        "knight", "queen", "king",
    ];
    const SUITS: [&str; 4] = ["swords", "wands", "pentacles", "cups"];

    // Retrieve the current time since the epoch in nanoseconds.
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let seconds = duration_since_epoch.as_secs();
    let seconds_hex = format!("{:x}", seconds);
    let nanos = duration_since_epoch.as_nanos();

    // Calculate the IX value.
    let mut ix = (nanos as usize) % 78;

    // Pick a hostname.
    if ix < MAJOR.len() {
        return format!("{:0>5.5}-{}", seconds_hex, MAJOR[ix]);
    }

    ix -= MAJOR.len();
    format!(
        "{:0>5.5}c-{}-of-{}",
        seconds_hex,
        MINOR[ix % MINOR.len()],
        SUITS[ix / MINOR.len()]
    )
}
