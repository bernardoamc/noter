use chrono::{Local, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;

const OSASCRIPT_TEMPLATE: &'static str = include_str!("../osascript_template");
const MAX_SCHEDULED_TIME_DIFF: i64 = 1;

#[derive(Serialize, Deserialize, Debug)]
enum Operation {
    Add { key: u64, value: Note },
    Remove { key: u64 },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    text: String,
    schedule: NaiveDateTime,
}

impl Note {
    pub fn new(text: String, datetime: Option<NaiveDateTime>, time: Option<NaiveTime>) -> Self {
        let schedule = datetime.unwrap_or_else(|| {
            Local::today()
                .and_time(time.unwrap())
                .unwrap()
                .naive_local()
        });

        Note { text, schedule }
    }
}

pub struct NoteStore {
    path: PathBuf,
    notes: HashMap<u64, Note>,
}

impl NoteStore {
    pub fn load(noter_path: impl Into<PathBuf>) -> Result<NoteStore, String> {
        let mut notes_map: HashMap<u64, Note> = HashMap::new();
        let noter_path = noter_path.into();

        let noter = OpenOptions::new().read(true).open(&noter_path).unwrap();
        let reader = BufReader::new(noter);
        let mut stream = Deserializer::from_reader(reader).into_iter::<Operation>();

        while let Some(operation) = stream.next() {
            let operation = operation.unwrap();

            match operation {
                Operation::Add { key, value } => {
                    notes_map.insert(key, value);
                }
                Operation::Remove { key } => {
                    notes_map.remove(&key);
                }
            }
        }

        Ok(NoteStore {
            notes: notes_map,
            path: noter_path,
        })
    }

    pub fn add(noter_path: impl Into<PathBuf>, note: Note) -> Result<(), String> {
        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&noter_path.into())
            .unwrap();

        let mut hasher = DefaultHasher::new();
        note.text.hash(&mut hasher);

        let add_op = Operation::Add {
            key: hasher.finish(),
            value: note,
        };
        serde_json::to_writer(&mut writer, &add_op).unwrap();

        Ok(())
    }

    pub fn check(&mut self) {
        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.path)
            .unwrap();

        let due_notes = self.notes.iter().filter(|&(_key, note)| is_note_due(note));

        for (key, note) in due_notes {
            Command::new("osascript")
                .args(&["-e", OSASCRIPT_TEMPLATE, &note.text])
                .output()
                .expect("failed to execute osascript");

            let remove_op = Operation::Remove { key: key.clone() };
            serde_json::to_writer(&mut writer, &remove_op).unwrap();
        }
    }
}

fn is_note_due(note: &Note) -> bool {
    let current_time = Local::now().naive_local();
    let difference_in_minutes = (note.schedule - current_time).num_minutes();

    difference_in_minutes < MAX_SCHEDULED_TIME_DIFF
}
