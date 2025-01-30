### Api Binary Analyzer

### Создание контекста с ссылки
- **Функция:** `create_analyzer_context_from_url`
- **Пример использования:**
```ts
const context: unknown | null = await invoke("create_analyzer_context_from_url", { url: "url" });
```

### Создание контекста из .json файла
- **Функция:** `create_analyzer_context`
- **Пример использования:**
```ts
const context: unknown | null = await invoke("create_analyzer_context_from_url", { path: "path/to/file.json" });
```

### Генерация контекста всех файлов из папки
- **Функция:** `generate_context`
- **Пример использования:**
```ts
const context: unknown | null = await invoke("generate_context", { path: "start_path" });
```

### Запуск анализа
- **Функция:** `run_analyzer`
- **Пример использования:**
```ts
await listen("analyzer_emit_event", (e) => { // прослушка ивентов анализа
    console.log(e.payload);
});

await invoke("run_analyzer", { context: context, startPath: "start_path" });
```