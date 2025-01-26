### Api Usn Journal

### Получить все диски
- **Функция:** `get_all_volumes`
- **Пример использования:**
```ts
enum Flag {
    SYSTEM,
    NTFS
}

type Volume = {
    path: string,
    free_space: number,
    available_space: number,
    total_space: number,
    flags: Flag[],
}

const volumes: Volume[] = await invoke("get_all_volumes");
```

### Получить записи из журнала
- **Функция:** `get_usn_journal_records`
- **Пример использования:**
```ts
enum Flag {
    SYSTEM,
    NTFS
}

type Volume = {
    path: string,
    free_space: number,
    available_space: number,
    total_space: number,
    flags: Flag[],
}

type FileRecord = {
    name: string,
    path: string | undefined,
    timestamp: number,
    reason: string,
}

const volume: Volume = ...;
const reason: number = -1; // все
const records: FileRecord[] = await invoke("get_usn_journal_records", { volume: volume, reason: reason });
```