use steam::{Steam, SteamAccount};

pub mod steam;
pub mod token;
pub mod tree;
pub mod tests;
pub mod convert;

#[tauri::command]
pub fn get_steam_accounts_history() -> Vec<SteamAccount> {
    let steam = Steam::new();
    steam.get_history_accounts()
}

#[tauri::command]
pub async fn is_vac_present(account: SteamAccount) -> bool {
    account.is_vac().await
}