use steam::{Steam, SteamAccount};

pub mod convert;
pub mod steam;
pub mod token;
pub mod tree;

#[tauri::command]
pub fn get_steam_accounts_history() -> Vec<SteamAccount> {
    let steam = Steam::new();
    steam.get_history_accounts()
}

#[tauri::command]
pub fn get_steam_avatar_cache() -> Vec<String> {
    let steam = Steam::new();
    steam.get_avatar_cache()
}

#[tauri::command]
pub async fn is_vac_present(account: SteamAccount) -> bool {
    account.is_vac().await
}
