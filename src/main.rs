mod logger;
mod db;

use clap::{Parser, Subcommand};
use chrono::{Local, DateTime};
use db::{init_db, CommandLog};

#[derive(Parser)]
#[command(name = "ctx")]
#[command(about = "Terminal command logger and productivity tracker", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log a command (internal use)
    LogCmd {
        command: String,
        cwd: String,
        exit_code: i32,
        duration_secs: f64,
    },
    /// Show complete command history
    Log {
        /// Show logs in reverse order (newest at top)
        #[arg(long)]
        reverse: bool,
        /// View logs with a pager (less)
        #[arg(long)]
        less: bool,
    },
    /// Show commands from the last 24 hours
    Today {
        /// Export a human-readable summary
        #[arg(long)]
        export: bool,
        /// Export in markdown format
        #[arg(long)]
        markdown: bool,
    },
    /// Show commands from the last 7 days
    Weekly {
        /// Export a human-readable summary
        #[arg(long)]
        export: bool,
        /// Export in markdown format
        #[arg(long)]
        markdown: bool,
    },
    /// Show summary for a specific project/folder
    Summary {
        folder: String,
    },
    /// Clear all command logs
    Clear,
    /// Show top N most used commands
    Top {
        #[arg(long, default_value_t = 10)]
        n: usize,
    },
    /// List all detected project folders with stats
    Projects,
    /// Search history for commands matching a pattern
    Search {
        pattern: String,
    },
    /// Show overall productivity stats
    Stats,
    /// Initialize shell integration
    Init,
}

fn main() {
    let db_path = dirs::home_dir().unwrap().join(".context/ctx.sqlite");
    let db_path_str = db_path.to_str().unwrap();
    let conn = init_db(db_path_str).expect("Failed to initialize database");

    let cli = Cli::parse();

    match cli.command {
        Commands::LogCmd { command, cwd, exit_code, duration_secs } => {
            let log = CommandLog {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Local::now(),
                cwd,
                command,
                exit_code,
                duration_secs,
            };
            db::insert_command_log(&conn, &log).expect("Failed to insert log");
        }
        Commands::Log { reverse, less } => {
            let order = if reverse { "DESC" } else { "ASC" };
            let query = format!("SELECT id, timestamp, cwd, command, exit_code, duration_secs FROM command_logs ORDER BY timestamp {}", order);
            let mut stmt = conn.prepare(&query).unwrap();
            let logs = stmt.query_map([], |row| {
                Ok(CommandLog {
                    id: row.get(0)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?).unwrap().with_timezone(&Local),
                    cwd: row.get(2)?,
                    command: row.get(3)?,
                    exit_code: row.get(4)?,
                    duration_secs: row.get(5)?,
                })
            }).unwrap();
            let mut output = String::new();
            for log in logs {
                let log = log.unwrap();
                output.push_str(&format!("[{}] {}\n  Dir: {}\n  Exit: {} | Duration: {:.2}s\n\n", log.timestamp, log.command, log.cwd, log.exit_code, log.duration_secs));
            }
            if less {
                use std::process::{Command, Stdio};
                let mut pager = Command::new("less")
                    .stdin(Stdio::piped())
                    .spawn()
                    .expect("Failed to launch less");
                use std::io::Write;
                if let Some(stdin) = pager.stdin.as_mut() {
                    stdin.write_all(output.as_bytes()).unwrap();
                }
                pager.wait().unwrap();
            } else {
                print!("{}", output);
            }
        }
        Commands::Today { export, markdown } => {
            use chrono::Duration;
            let now = Local::now();
            let since = now - Duration::hours(24);
            let mut stmt = conn.prepare("SELECT timestamp, cwd, command, duration_secs FROM command_logs WHERE timestamp >= ?1 ORDER BY timestamp ASC").unwrap();
            let logs = stmt.query_map([since.to_rfc3339()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, f64>(3)?))
            }).unwrap();
            let mut total_commands = 0;
            let mut total_time = 0.0;
            use std::collections::HashMap;
            let mut folder_time: HashMap<String, f64> = HashMap::new();
            let mut command_count: HashMap<String, usize> = HashMap::new();
            let mut first_timestamp: Option<DateTime<Local>> = None;
            let mut last_timestamp: Option<DateTime<Local>> = None;
            for log in logs {
                let (timestamp, cwd, command, duration): (String, String, String, f64) = log.unwrap();
                let trimmed = command.trim_start();
                if trimmed == "ctx" || trimmed.starts_with("ctx ") { continue; }
                let ts = DateTime::parse_from_rfc3339(&timestamp).unwrap().with_timezone(&Local);
                if first_timestamp.is_none() { first_timestamp = Some(ts); }
                last_timestamp = Some(ts);
                *folder_time.entry(cwd.clone()).or_insert(0.0) += duration;
                *command_count.entry(command.clone()).or_insert(0) += 1;
                total_commands += 1;
                total_time += duration;
            }
            if export || markdown {
                let mut folders: Vec<_> = folder_time.into_iter().collect();
                folders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                let top_folders: Vec<_> = folders.into_iter().take(3).collect();
                let mut commands: Vec<_> = command_count.into_iter().collect();
                commands.sort_by(|a, b| b.1.cmp(&a.1));
                if markdown {
                    println!("## Productivity Summary (Today)");
                    println!("- **Total commands:** {}", total_commands);
                    println!("- **Total terminal time:** {:.2} seconds", total_time);
                    if let (Some(first), Some(last)) = (first_timestamp, last_timestamp) {
                        let uptime = last.signed_duration_since(first).num_seconds();
                        println!("- **Total terminal uptime:** {} seconds", uptime);
                    } else {
                        println!("- **Total terminal uptime:** N/A");
                    }
                    println!("- **Top 3 most worked folders:**");
                    for (i, (folder, time)) in top_folders.iter().enumerate() {
                        println!("  {}. {} (`{:.2}` seconds)", i + 1, folder, time);
                    }
                    println!("- **Top 3 most used commands:**");
                    for (i, (cmd, count)) in commands.iter().take(3).enumerate() {
                        println!("  {}. `{}` ({} times)", i + 1, cmd, count);
                    }
                } else {
                    println!("Productivity Summary (Today):");
                    println!("Total commands: {}", total_commands);
                    println!("Total terminal time: {:.2} seconds", total_time);
                    if let (Some(first), Some(last)) = (first_timestamp, last_timestamp) {
                        let uptime = last.signed_duration_since(first).num_seconds();
                        println!("Total terminal uptime: {} seconds", uptime);
                    } else {
                        println!("Total terminal uptime: N/A");
                    }
                    println!("Top 3 most worked folders:");
                    for (i, (folder, time)) in top_folders.iter().enumerate() {
                        println!("  {}. {} ({:.2} seconds)", i + 1, folder, time);
                    }
                    println!("Top 3 most used commands:");
                    for (i, (cmd, count)) in commands.iter().take(3).enumerate() {
                        println!("  {}. {} ({} times)", i + 1, cmd, count);
                    }
                }
            } else {
                let mut stmt = conn.prepare("SELECT id, timestamp, cwd, command, exit_code, duration_secs FROM command_logs WHERE timestamp >= ?1 ORDER BY timestamp ASC").unwrap();
                let logs = stmt.query_map([since.to_rfc3339()], |row| {
                    Ok(CommandLog {
                        id: row.get(0)?,
                        timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?).unwrap().with_timezone(&Local),
                        cwd: row.get(2)?,
                        command: row.get(3)?,
                        exit_code: row.get(4)?,
                        duration_secs: row.get(5)?,
                    })
                }).unwrap();
                for log in logs {
                    let log = log.unwrap();
                    println!("[{}] {}\n  Dir: {}\n  Exit: {} | Duration: {:.2}s\n", log.timestamp, log.command, log.cwd, log.exit_code, log.duration_secs);
                }
            }
        }
        Commands::Weekly { export, markdown } => {
            use chrono::Duration;
            let now = Local::now();
            let since = now - Duration::days(7);
            let mut stmt = conn.prepare("SELECT timestamp, cwd, command, duration_secs FROM command_logs WHERE timestamp >= ?1 ORDER BY timestamp ASC").unwrap();
            let logs = stmt.query_map([since.to_rfc3339()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, f64>(3)?))
            }).unwrap();
            let mut total_commands = 0;
            let mut total_time = 0.0;
            use std::collections::HashMap;
            let mut folder_time: HashMap<String, f64> = HashMap::new();
            let mut command_count: HashMap<String, usize> = HashMap::new();
            let mut first_timestamp: Option<DateTime<Local>> = None;
            let mut last_timestamp: Option<DateTime<Local>> = None;
            for log in logs {
                let (timestamp, cwd, command, duration): (String, String, String, f64) = log.unwrap();
                let trimmed = command.trim_start();
                if trimmed == "ctx" || trimmed.starts_with("ctx ") { continue; }
                let ts = DateTime::parse_from_rfc3339(&timestamp).unwrap().with_timezone(&Local);
                if first_timestamp.is_none() { first_timestamp = Some(ts); }
                last_timestamp = Some(ts);
                *folder_time.entry(cwd.clone()).or_insert(0.0) += duration;
                *command_count.entry(command.clone()).or_insert(0) += 1;
                total_commands += 1;
                total_time += duration;
            }
            if export || markdown {
                let mut folders: Vec<_> = folder_time.into_iter().collect();
                folders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                let top_folders: Vec<_> = folders.into_iter().take(3).collect();
                let mut commands: Vec<_> = command_count.into_iter().collect();
                commands.sort_by(|a, b| b.1.cmp(&a.1));
                if markdown {
                    println!("## Productivity Summary (Weekly)");
                    println!("- **Total commands:** {}", total_commands);
                    println!("- **Total terminal time:** {:.2} seconds", total_time);
                    if let (Some(first), Some(last)) = (first_timestamp, last_timestamp) {
                        let uptime = last.signed_duration_since(first).num_seconds();
                        println!("- **Total terminal uptime:** {} seconds", uptime);
                    } else {
                        println!("- **Total terminal uptime:** N/A");
                    }
                    println!("- **Top 3 most worked folders:**");
                    for (i, (folder, time)) in top_folders.iter().enumerate() {
                        println!("  {}. {} (`{:.2}` seconds)", i + 1, folder, time);
                    }
                    println!("- **Top 3 most used commands:**");
                    for (i, (cmd, count)) in commands.iter().take(3).enumerate() {
                        println!("  {}. `{}` ({} times)", i + 1, cmd, count);
                    }
                } else {
                    println!("Productivity Summary (Weekly):");
                    println!("Total commands: {}", total_commands);
                    println!("Total terminal time: {:.2} seconds", total_time);
                    if let (Some(first), Some(last)) = (first_timestamp, last_timestamp) {
                        let uptime = last.signed_duration_since(first).num_seconds();
                        println!("Total terminal uptime: {} seconds", uptime);
                    } else {
                        println!("Total terminal uptime: N/A");
                    }
                    println!("Top 3 most worked folders:");
                    for (i, (folder, time)) in top_folders.iter().enumerate() {
                        println!("  {}. {} ({:.2} seconds)", i + 1, folder, time);
                    }
                    println!("Top 3 most used commands:");
                    for (i, (cmd, count)) in commands.iter().take(3).enumerate() {
                        println!("  {}. {} ({} times)", i + 1, cmd, count);
                    }
                }
            } else {
                let mut stmt = conn.prepare("SELECT id, timestamp, cwd, command, exit_code, duration_secs FROM command_logs WHERE timestamp >= ?1 ORDER BY timestamp DESC").unwrap();
                let logs = stmt.query_map([since.to_rfc3339()], |row| {
                    Ok(CommandLog {
                        id: row.get(0)?,
                        timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?).unwrap().with_timezone(&Local),
                        cwd: row.get(2)?,
                        command: row.get(3)?,
                        exit_code: row.get(4)?,
                        duration_secs: row.get(5)?,
                    })
                }).unwrap();
                for log in logs {
                    let log = log.unwrap();
                    println!("[{}] {}\n  Dir: {}\n  Exit: {} | Duration: {:.2}s\n", log.timestamp, log.command, log.cwd, log.exit_code, log.duration_secs);
                }
            }
        }
        Commands::Summary { folder } => {
            let mut stmt = conn.prepare("SELECT COUNT(*), SUM(duration_secs) FROM command_logs WHERE cwd LIKE ?1").unwrap();
            let like_pattern = format!("%{}%", folder);
            let mut rows = stmt.query([like_pattern]).unwrap();
            if let Some(row) = rows.next().unwrap() {
                let count: i64 = row.get(0).unwrap_or(0);
                let total_time: f64 = row.get(1).unwrap_or(0.0);
                println!("Summary for '{}':\n  Commands run: {}\n  Total time spent: {:.2} seconds", folder, count, total_time);
            } else {
                println!("No data found for project/folder '{}'.", folder);
            }
        }
        Commands::Clear => {
            use std::io::{self, Write};
            print!("Are you sure you want to clear all logs? [y/N]: ");
            io::stdout().flush().unwrap();
            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();
            if answer.trim().eq_ignore_ascii_case("y") {
                conn.execute("DELETE FROM command_logs", []).expect("Failed to clear logs");
                println!("All logs have been cleared.");
            } else {
                println!("Aborted. No logs were cleared.");
            }
        }
        Commands::Top { n } => {
            let mut stmt = conn.prepare("SELECT command, COUNT(*) as cnt FROM command_logs WHERE command NOT LIKE 'ctx%' GROUP BY command ORDER BY cnt DESC LIMIT ?1").unwrap();
            let rows = stmt.query_map([n as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            }).unwrap();
            println!("Top {} most used commands:", n);
            for (i, row) in rows.enumerate() {
                let (cmd, count) = row.unwrap();
                println!("  {}. {} ({} times)", i + 1, cmd, count);
            }
        }
        Commands::Projects => {
            let mut stmt = conn.prepare("SELECT cwd, COUNT(*), SUM(duration_secs) FROM command_logs WHERE command NOT LIKE 'ctx%' GROUP BY cwd ORDER BY COUNT(*) DESC").unwrap();
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, f64>(2)?))
            }).unwrap();
            println!("Project folders:");
            for (i, row) in rows.enumerate() {
                let (cwd, count, total_time) = row.unwrap();
                println!("  {}. {} ({} commands, {:.2} seconds)", i + 1, cwd, count, total_time);
            }
        }
        Commands::Search { pattern } => {
            let like_pattern = format!("%{}%", pattern);
            let mut stmt = conn.prepare("SELECT timestamp, cwd, command, exit_code, duration_secs FROM command_logs WHERE command LIKE ?1 AND command NOT LIKE 'ctx%' ORDER BY timestamp ASC").unwrap();
            let rows = stmt.query_map([like_pattern], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i32>(3)?, row.get::<_, f64>(4)?))
            }).unwrap();
            println!("Search results for '{}':", pattern);
            for row in rows {
                let (timestamp, cwd, command, exit_code, duration) = row.unwrap();
                println!("[{}] {}\n  Dir: {}\n  Exit: {} | Duration: {:.2}s\n", timestamp, command, cwd, exit_code, duration);
            }
        }
        Commands::Stats => {
            let mut stmt = conn.prepare("SELECT COUNT(*), SUM(duration_secs), MIN(duration_secs), MAX(duration_secs), AVG(duration_secs) FROM command_logs WHERE command NOT LIKE 'ctx%'").unwrap();
            let mut rows = stmt.query([]).unwrap();
            if let Some(row) = rows.next().unwrap() {
                let total: i64 = row.get(0).unwrap_or(0);
                let sum: f64 = row.get(1).unwrap_or(0.0);
                let min: f64 = row.get(2).unwrap_or(0.0);
                let max: f64 = row.get(3).unwrap_or(0.0);
                let avg: f64 = row.get(4).unwrap_or(0.0);
                println!("Overall Productivity Stats:");
                println!("  Total commands: {}", total);
                println!("  Total terminal time: {:.2} seconds", sum);
                println!("  Shortest command: {:.2} seconds", min);
                println!("  Longest command: {:.2} seconds", max);
                println!("  Average command duration: {:.2} seconds", avg);
            }
        }
        Commands::Init => {
            use std::env;
            use std::fs;
            use std::io::{self, Write};
            #[cfg(target_os = "macos")]
            fn get_shell() -> String {
                // Try ZSH/FISH env vars first
                if std::env::var("ZSH_VERSION").is_ok() {
                    return "zsh".to_string();
                }
                if std::env::var("FISH_VERSION").is_ok() {
                    return "fish".to_string();
                }
                // Use ps to get current shell
                if let Ok(output) = std::process::Command::new("ps").args(["-p", &std::process::id().to_string(), "-o", "comm="]).output() {
                    let comm = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if comm == "zsh" || comm == "fish" || comm == "bash" {
                        return comm;
                    }
                }
                std::env::var("SHELL").unwrap_or_default()
            }
            #[cfg(not(target_os = "macos"))]
            fn get_shell() -> String {
                let mut shell = std::env::var("SHELL").unwrap_or_default();
                if std::env::var("ZSH_VERSION").is_ok() {
                    shell = "zsh".to_string();
                } else if std::env::var("FISH_VERSION").is_ok() {
                    shell = "fish".to_string();
                } else if let Ok(stat) = fs::read_to_string("/proc/self/stat") {
                    let parts: Vec<&str> = stat.split_whitespace().collect();
                    if parts.len() > 1 {
                        let ppid = parts[3];
                        if let Ok(ppid_stat) = fs::read_to_string(format!("/proc/{}/comm", ppid)) {
                            let comm = ppid_stat.trim();
                            if comm == "zsh" || comm == "fish" || comm == "bash" {
                                shell = comm.to_string();
                            }
                        }
                    }
                }
                shell
            }
            #[cfg(target_os = "macos")]
            let time_cmd = "date +%s";
            #[cfg(not(target_os = "macos"))]
            let time_cmd = "date +%s%N";
            let shell = get_shell();
            let mut snippet = String::new();
            let mut config_path = String::new();
            if shell.contains("zsh") {
                snippet = format!("function ctx_preexec() {{\n    export CTX_CMD_START_TIME=$({})\n    export CTX_CMD_TO_LOG=\"$1\"\n}}\nfunction ctx_precmd() {{\n    if [[ -n \"$CTX_CMD_START_TIME\" && -n \"$CTX_CMD_TO_LOG\" ]]; then\n        local end_time=$({})\n        local duration_ns=$((end_time - CTX_CMD_START_TIME))\n        local duration_s=$(awk \"BEGIN {{print $duration_ns/1000000000}}\")\n        local exit_code=$?\n        if [[ ! \"$CTX_CMD_TO_LOG\" =~ ^ctx($|[[:space:]]) ]]; then\n            ctx log-cmd \"$CTX_CMD_TO_LOG\" \"$PWD\" \"$exit_code\" \"$duration_s\"\n        fi\n        unset CTX_CMD_START_TIME\n        unset CTX_CMD_TO_LOG\n    fi\n}}\nautoload -Uz add-zsh-hook\nadd-zsh-hook preexec ctx_preexec\nadd-zsh-hook precmd ctx_precmd\n", time_cmd, time_cmd);
                config_path = format!("{}/.zshrc", env::var("HOME").unwrap());
            } else if shell.contains("fish") {
                snippet = format!("function ctx_preexec --on-event fish_preexec\n    set -g CTX_CMD_START_TIME ({} )\n    set -g CTX_CMD_TO_LOG $argv[1]\nend\n\nfunction ctx_precmd --on-event fish_prompt\n    if test -n \"$CTX_CMD_START_TIME\" -a -n \"$CTX_CMD_TO_LOG\"\n        set end_time ({} )\n        set duration_ns (math $end_time - $CTX_CMD_START_TIME)\n        set duration_s (math --scale 2 $duration_ns / 1000000000)\n        set exit_code $status\n        if not string match -r '^ctx($|\\s)' -- $CTX_CMD_TO_LOG\n            ctx log-cmd \"$CTX_CMD_TO_LOG\" \"$PWD\" \"$exit_code\" \"$duration_s\"\n        end\n        set -e CTX_CMD_START_TIME\n        set -e CTX_CMD_TO_LOG\n    end\nend\n", time_cmd, time_cmd);
                config_path = format!("{}/.config/fish/config.fish", env::var("HOME").unwrap());
            } else {
                snippet = format!("[[ -f ~/.bash-preexec.sh ]] && source ~/.bash-preexec.sh\n\nfunction ctx_preexec() {{\n    export CTX_CMD_START_TIME=$({})\n    export CTX_CMD_TO_LOG=\"$1\"\n}}\nfunction ctx_precmd() {{\n    if [ -n \"$CTX_CMD_START_TIME\" ] && [ -n \"$CTX_CMD_TO_LOG\" ]; then\n        local end_time=$({})\n        local duration_ns=$((end_time - CTX_CMD_START_TIME))\n        local duration_s=$(awk \"BEGIN {{print $duration_ns/1000000000}}\")\n        local exit_code=$?\n        if [[ ! \"$CTX_CMD_TO_LOG\" =~ ^ctx($|[[:space:]]) ]]; then\n            ctx log-cmd \"$CTX_CMD_TO_LOG\" \"$PWD\" \"$exit_code\" \"$duration_s\"\n        fi\n        unset CTX_CMD_START_TIME\n        unset CTX_CMD_TO_LOG\n    fi\n}}\npreexec_functions+=(ctx_preexec)\nprecmd_functions+=(ctx_precmd)\n", time_cmd, time_cmd);
                config_path = format!("{}/.bashrc", env::var("HOME").unwrap());
            }
            println!("# The following snippet will enable ctx logging for your shell:\n\n{}", snippet);
            #[cfg(target_os = "macos")]
            println!("\n**Note for macOS users:** For nanosecond precision, install GNU coreutils and use 'gdate' instead of 'date'.\nE.g., replace 'date +%s' with 'gdate +%s%N' in the snippet above after installing coreutils with 'brew install coreutils'.");
            print!("\nWould you like to append this to {}? [y/N]: ", config_path);
            io::stdout().flush().unwrap();
            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();
            if answer.trim().eq_ignore_ascii_case("y") {
                use std::fs::OpenOptions;
                let mut file = OpenOptions::new().create(true).append(true).open(&config_path).unwrap();
                writeln!(file, "\n# ctx shell integration\n{}", snippet).unwrap();
                println!("Appended to {}!", config_path);
                println!("\nTo activate ctx logging, run: source {}", config_path);
            } else {
                println!("Not appended. You can manually add the snippet above to your shell config file.");
            }
        }
    }
}
