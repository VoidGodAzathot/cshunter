use mini_dat::{MiniDat, MiniDatEmployee, MiniDatInfo};
use registry_md::{Radar, SevenZip, UserAssist, WinRar};

pub mod mini_dat;
pub mod registry_md;

pub const MINI_DAT_META: [MiniDatInfo; 4] = [
    MiniDatInfo {
        id: "radar",
        name: "Отсканированные файлы",
        description: "Почти всегда файлы просканированные внутренней функцией антивируса системы Windows, данная проверка как правило выполняется перед запуском.",
    },
    MiniDatInfo {
        id: "user_assist",
        name: "Данные об активности",
        description: "В частности о запуске программ через Проводник и меню «Пуск». Этот раздел используется для сбора информации о часто используемых приложениях, что помогает операционной системе адаптировать пользовательский интерфейс.",
    },
    MiniDatInfo {
        id: "seven_zip",
        name: "Использование архивов 7Zip",
        description: "Произведенные когда либо любые действия с архивами через 7Zip File Manager.",
    },
    MiniDatInfo {
        id: "winrar",
        name: "Использование архивов WinRar",
        description: "Произведенные когда либо любые действия с архивом .rar (в основном) через WinRar Archive.",
    },
];

#[tauri::command]
pub fn collect_mini_dat() -> Vec<MiniDat> {
    let mut employees: Vec<MiniDat> = vec![];

    employees.append(&mut WinRar::run());
    employees.append(&mut SevenZip::run());
    employees.append(&mut UserAssist::run());
    employees.append(&mut Radar::run());

    employees
}

#[tauri::command]
pub fn get_mini_dat_info(id: String) -> Option<MiniDatInfo> {
    for info in MINI_DAT_META {
        if info.id.eq(&id) {
            return Some(info);
        }
    }

    None
}
