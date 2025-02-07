export function dateFromWebkit(webkitTimestamp: number): string {
    const epochStart = new Date(Date.UTC(1601, 0, 1));
    const milliseconds = webkitTimestamp / 1_000;
    const resultDate = new Date(epochStart.getTime() + milliseconds);
    return resultDate.toLocaleString();
}

export function dateFromUsn(timestamp: number): string {
    const EPOCH_DIFF = 11644473600000;
    const TICKS_PER_MILLISECOND = 10000;
    const unixTimeMs = Number((timestamp / TICKS_PER_MILLISECOND) - EPOCH_DIFF);
    return new Date(unixTimeMs).toLocaleString();
}

export function dateFromUnix(timestamp: number): string {
    const resultDate = new Date(timestamp * 1000);
    return resultDate.toLocaleString();
}

export function jsonToType<T>(value: string): T {
    return JSON.parse(value) as T;
}

export async function filterIsPresent<T>(filter: string, data: T): Promise<boolean> {
    const filterWords = filter.split('||').map(word => word.trim().toLowerCase());

    const checkValue = (value: any): boolean => {
        if (value === null || value === undefined) return false;

        if (typeof value === 'object') {
            return Array.isArray(value)
                ? value.some(item => checkValue(item))
                : Object.values(value).some(nestedValue => checkValue(nestedValue));
        }

        const stringValue = String(value).toLowerCase();
        return filterWords.some(word => stringValue.includes(word));
    };

    return checkValue(data);
}

export async function asyncFilter<T>(arr: T[], cb: (el: T) => Promise<boolean>): Promise<T[]> {
    const filtered: T[] = [];

    for (const element of arr) {
        const needAdd = await cb(element);

        if (needAdd) {
            filtered.push(element);
        }
    }

    return filtered;
}