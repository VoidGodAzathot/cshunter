use mini_dat::{MiniDat, MiniDatEmployee, MiniDatInfo};
use registry_md::{AppCompatCache, AppSwitched, Bam, Radar, SevenZip, UserAssist, WinRar};

pub mod mini_dat;
pub mod registry_md;

pub const MINI_DAT_META: [MiniDatInfo; 7] = [
    MiniDatInfo {
        id: "radar",
        name: "Отсканированные файлы",
        description: "Почти всегда файлы просканированные внутренней функцией антивируса системы Windows, данная проверка как правило выполняется перед запуском.",
        filtering: true,
        stable: true
    },
    MiniDatInfo {
        id: "user_assist",
        name: "Данные об активности",
        description: "В частности о запуске программ через Проводник и меню «Пуск». Этот раздел используется для сбора информации о часто используемых приложениях, что помогает операционной системе адаптировать пользовательский интерфейс.",
        filtering: true,
        stable: true
    },
    MiniDatInfo {
        id: "seven_zip",
        name: "Использование архивов 7Zip",
        description: "Произведенные когда либо любые действия с архивами через 7Zip File Manager.",
        filtering: true,
        stable: true
    },
    MiniDatInfo {
        id: "winrar",
        name: "Использование архивов WinRar",
        description: "Произведенные когда либо любые действия с архивом .rar (в основном) через WinRar Archive.",
        filtering: true,
        stable: true
    },
    MiniDatInfo {
        id: "app_compat_cache",
        name: "Кэш совместимости",
        description: "Хранится кэш совместимости приложений, также известный как Application Compatibility Cache. Cодержит список исполняемых файлов .exe, которые запускались на системе.",
        filtering: true,
        stable: true
    },
    MiniDatInfo {
        id: "bam",
        name: "Мониторинг фоновой активности",
        description: "Cодержит информацию о процессах, которые выполнялись в системе, включая их активность в фоновом режиме, также известный как Background Activity Moderator.",
        filtering: true,
        stable: true
    },
    MiniDatInfo {
        id: "app_switched",
        name: "Статистика переключения между приложениями",
        description: "Хранится статистика использования функции переключения между приложениями.",
        filtering: true,
        stable: true
    }
];

#[tauri::command]
pub fn collect_mini_dat() -> Vec<MiniDat> {
    let mut employees: Vec<MiniDat> = vec![];

    employees.append(&mut WinRar::run());
    employees.append(&mut SevenZip::run());
    employees.append(&mut UserAssist::run());
    employees.append(&mut Radar::run());
    employees.append(&mut AppCompatCache::run());
    employees.append(&mut Bam::run());
    employees.append(&mut AppSwitched::run());

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
