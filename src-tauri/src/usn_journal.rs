use usn_journal::UsnJournal;
use usn_record::FileRecord;
use volume::Volume;

pub mod tests;
pub mod usn_journal;
pub mod usn_record;
pub mod volume;

#[tauri::command]
pub fn get_all_volumes() -> Vec<Volume> {
    volume::get_all_volumes()
}

#[tauri::command]
pub fn get_usn_journal_records(volume: Volume, reason: i32) -> Vec<FileRecord> {
    let mut journal = UsnJournal::new(volume);

    if journal.init() {
        let records = if reason >= 0 {
            journal.read(reason as u32)
        } else {
            journal.read_all()
        };

        drop(journal);

        return records;
    }

    vec![]
}
