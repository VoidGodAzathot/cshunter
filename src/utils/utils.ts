export function dateFromWebkit(webkitTimestamp: number): string {
    const epochStart = new Date(Date.UTC(1601, 0, 1));
    const milliseconds = webkitTimestamp / 1_000;
    const resultDate = new Date(epochStart.getTime() + milliseconds);
    return resultDate.toLocaleString();
}

export function filterIsPresent<T>(filter: string, data: T): boolean {
    const filterValues = filter.split("||").map(item => item.trim().toLowerCase());
    if (filterValues.length === 0 || filterValues[0].length === 0) return true;

    const collectValues = (input: unknown): string[] => {
        if (typeof input === "object" && input !== null) {
            return Array.isArray(input) 
                ? input.flatMap(collectValues)
                : Object.values(input).flatMap(collectValues);
        }
        return [String(input).toLowerCase().trim()];
    };

    const dataValues = collectValues(data);
    
    return filterValues.some(filterValue => 
        dataValues.some(dataValue => dataValue.includes(filterValue))
    );
}