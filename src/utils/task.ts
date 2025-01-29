export type Task = {
    name: string,
    id: string,
    worker: () => Promise<void>
}

