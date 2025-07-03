use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};

// Path to store last command info for debounce and duplicate detection
fn ctx_state_path() -> std::path::PathBuf {
    dirs::home_dir().unwrap().join(".context/ctx_state")
}

pub fn log_command(args: Vec<String>) {
    // (4) Only log if attached to a TTY
    if !atty::is(atty::Stream::Stdout) {
        return;
    }

    let start = Instant::now();
    let command = &args[0];
    let cmd_args = &args[1..];

    // (2) Blacklist certain commands
    let blacklist = ["ls", "clear", "pwd", "history", "exit"];
    if blacklist.iter().any(|&b| command == b) {
        return;
    }

    // (3) Ignore consecutive duplicate commands
    let state_path = ctx_state_path();
    let mut last_cmd = String::new();
    let mut last_time = 0u64;
    if let Ok(file) = fs::File::open(&state_path) {
        let mut reader = BufReader::new(file);
        let mut buf = String::new();
        if reader.read_line(&mut buf).is_ok() {
            last_cmd = buf.trim().to_string();
        }
        buf.clear();
        if reader.read_line(&mut buf).is_ok() {
            last_time = buf.trim().parse().unwrap_or(0);
        }
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let now_u64 = now as u64;
    let full_command = args.join(" ");
    if full_command == last_cmd {
        return;
    }
    // (1) Debounce: Ignore if <0.5s since last command
    if now - (last_time as f64) < 0.5 {
        return;
    }
    // Save current command and time
    if let Ok(mut file) = OpenOptions::new().create(true).write(true).truncate(true).open(&state_path) {
        writeln!(file, "{}", full_command).ok();
        writeln!(file, "{}", now_u64).ok();
    }

    let status = Command::new(command)
        .args(cmd_args)
        .status()
        .expect("Failed to run command");

    let duration = start.elapsed().as_secs_f64();
    let cwd = env::current_dir().unwrap().to_str().unwrap().to_string();
    let exit_code = status.code().unwrap_or(-1);

    let _ = Command::new(env::current_exe().unwrap())
        .arg("log-cmd")
        .arg("--command").arg(full_command)
        .arg("--cwd").arg(cwd)
        .arg("--exit-code").arg(exit_code.to_string())
        .arg("--duration-secs").arg(duration.to_string())
        .status();
}
