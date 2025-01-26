#[cfg(test)]
mod tests {
    use windows::Win32::System::Ioctl::USN_REASON_FILE_CREATE;

    use crate::usn_journal::{usn_journal::UsnJournal, volume::get_all_volumes};

    #[test]
    fn main_test() {
        println!("!");
    }

    #[test]
    fn volumes_test() {
        for volume in get_all_volumes() {
            println!("{:?}", volume.flags);
        }
    }

    #[test]
    fn usn_test() {
        // создаем файл
        for volume in get_all_volumes() {
            let mut journal = UsnJournal::new(volume);

            if journal.init() {
                let response = journal.read(USN_REASON_FILE_CREATE);
                println!("{}", response.len());
                drop(journal);
            }
        }
    }
}