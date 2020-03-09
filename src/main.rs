use note_store::{Note, NoteStore};
mod note_store;

use chrono::{NaiveDateTime, NaiveTime};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use structopt::StructOpt;

const PLIST_TEMPLATE: &'static str = include_str!("../plist_template");

fn parse_date(date_input: &str) -> chrono::ParseResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(date_input, "%F %R")
}

fn parse_time(time_input: &str) -> chrono::ParseResult<NaiveTime> {
    NaiveTime::parse_from_str(time_input, "%H:%M")
}

#[derive(Debug, StructOpt)]
enum CommandOption {
    #[structopt(name = "add")]
    /// Adds the specific note to the list
    Add {
        #[structopt(name = "NOTE")]
        note: String,

        /// Set date in the format yy-MM-dd hh:mm
        #[structopt(short, long, parse(try_from_str = "parse_date"))]
        datetime: Option<NaiveDateTime>,

        /// Set time in the format HH:MM (default to today)
        #[structopt(
            short,
            long,
            required_unless = "datetime",
            parse(try_from_str = "parse_time")
        )]
        time: Option<NaiveTime>,
    },
    #[structopt(name = "check")]
    /// Checks which notes are due
    Check {},
}

fn main() {
    let options = CommandOption::from_args();
    setup_launchd();
    run(options);
}

fn setup_launchd() {
    let mut launch_agents_path = dirs::home_dir().unwrap();
    launch_agents_path.push("Library/LaunchAgents/com.rust.noter.plist");

    if !Path::new(launch_agents_path.as_path()).exists() {
        let mut plist_writer = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&launch_agents_path)
            .unwrap();
        plist_writer.write_all(PLIST_TEMPLATE.as_bytes()).unwrap();
        plist_writer.flush().unwrap();

        Command::new("launchctl")
            .args(&["load", launch_agents_path.as_path().to_str().unwrap()])
            .output()
            .expect("failed to execute launchctl load");
    }
}

fn run(options: CommandOption) {
    let mut noter_path = dirs::home_dir().unwrap();
    noter_path.push(".noter");

    match options {
        CommandOption::Add {
            note,
            datetime,
            time,
        } => {
            let new_note = Note::new(note, datetime, time);
            NoteStore::add(noter_path, new_note).unwrap();
        }
        CommandOption::Check {} => {
            let mut note_store = NoteStore::load(noter_path).unwrap();
            note_store.check();
        }
    }
}
