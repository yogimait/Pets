export const NOW = Date.now();

export const minutesAgo = (minutes: number): number => NOW - minutes * 60_000;
export const hoursAgo = (hours: number): number => NOW - hours * 3_600_000;
export const daysAgo = (days: number): number => NOW - days * 86_400_000;
