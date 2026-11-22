//! Public interface of the notes feature.
use super::logic::coordinator::Notes;
use super::persistence::store::{NoteInfo, Store};
use super::ui::note_window;
use crate::core::contracts::Feature;

pub fn create() -> Box<dyn Feature> {
    Box::new(Notes)
}

pub type Info = NoteInfo;

pub fn list_notes() -> Vec<NoteInfo> {
    Store::default_store().map(|s| s.list()).unwrap_or_default()
}

pub fn rename_note(old: &str, new: &str) -> Result<String, String> {
    let s = Store::default_store().ok_or("notes folder is unavailable")?;
    if note_window::current_name().as_deref() == Some(old) {
        note_window::flush();
    }
    let n = s.rename(old, new)?;
    note_window::renamed(old, &n);
    Ok(n)
}

pub fn delete_note(name: &str) -> Result<(), String> {
    let s = Store::default_store().ok_or("notes folder is unavailable")?;
    note_window::forget(name);
    s.delete(name)
}

pub fn open_note(name: &str) -> Result<(), String> {
    note_window::open(name)
}
