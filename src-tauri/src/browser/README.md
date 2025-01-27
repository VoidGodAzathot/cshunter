### Api Browsers Dat

### Получить поддерживаемые браузеры
- **Функция:** `get_supported_browsers`
- **Пример использования:**
```ts
enum Driver {
    BLINK
}

type Browser = {
    id: string,
    path: string,
    driver: Driver,
}

const browsers: Browser[] = await invoke("get_supported_browsers");
```

### Получить данные браузера
- **Функции:** `get_browser_visit_data, get_browser_cache_data, get_browser_download_data`
- **Пример использования:**
```ts
type DownloadDat = {
    browser: string,
    file: string,
    url: string,
    imestamp: number
}

type VisitDat = {
    browser: string,
    title: string,
    url: string,
    timestamp: number
}

type CacheDat = {
    browser: string,
    url: string
}

const browser: Browser = ...;
const visit_dat: VisitDat[] = await invoke("get_browser_visit_data", { browserId: browser.id });
const cache_dat: CacheDat[] = await invoke("get_browser_cache_data", { browserId: browser.id });
const download_dat: DownloadDat[] = await invoke("get_browser_download_data", { browserId: browser.id });
```