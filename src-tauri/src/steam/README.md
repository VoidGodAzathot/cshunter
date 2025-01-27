### Api Steam Accounts

### Получить историю аккаунтов
- **Функция:** `get_steam_accounts_history`
- **Пример использования:**
```ts
type SteamAccount = {
    id: number,
    persona_name: string,
    account_name: string,
    timestamp: string,
    most_recent: string,
}

const accounts: SteamAccount[] = await invoke("get_steam_accounts_history");
```

### Проверка на VAC
- **Функция:** `is_vac_present`
- **Пример использования:**
```ts
type SteamAccount = {
    id: number,
    persona_name: string,
    account_name: string,
    timestamp: string,
    most_recent: string,
}

const account: SteamAccount = ...;
const vac: bool = await invoke("is_vac_present". { account: account });
```