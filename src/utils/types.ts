export enum Flag {
    SYSTEM,
    NTFS
}

export type Volume = {
    path: string,
    free_space: number,
    available_space: number,
    total_space: number,
    flags: Flag[],
}

export type FileRecord = {
    name: string,
    path: string | undefined,
    timestamp: number,
    reason: string,
}

export type SteamAccount = {
    id: number,
    persona_name: string,
    account_name: string,
    timestamp: string,
    most_recent: string,
}

export enum Driver {
    BLINK
}

export type Browser = {
    id: string,
    path: string,
    driver: Driver,
    support: boolean
}

export type DownloadDat = {
    browser: string,
    file: string,
    url: string,
    imestamp: number
}

export type VisitDat = {
    browser: string,
    title: string,
    url: string,
    timestamp: number
}

export type CacheDat = {
    browser: string,
    url: string
}

export type AnalyzeContext = {
    items: ItemContext[]
}

export type ItemContext = {
    name: string,
    size: number,
    crc32: number[],
    tls: number
}

export type Page = {
    source: () => JSX.Element,
    icon: JSX.Element,
    name: string
}