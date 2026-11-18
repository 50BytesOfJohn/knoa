//! When an event happens, read from its `starts` and `ends` fields, as knoa_core's `event`
//! module reads them, and placed in the viewer's local time.

/** One `datetime` field value. Without `time` it is all-day. */
export interface EventTime {
	year: number;
	/** 1 to 12. */
	month: number;
	day: number;
	time?: {
		hour: number;
		minute: number;
		/** Minutes east of UTC. Without one the time is floating: the same wall time anywhere. */
		offsetMinutes?: number;
	};
}

/**
 * An event in the viewer's local time. `end` is exclusive and never before `start`; when
 * `endKnown` is false the page gave no usable end and `end` is only the end of the start's
 * day, so show no end time.
 */
export interface EventSpan {
	start: Date;
	end: Date;
	/** Whether `end` is the page's own end, read and not before `start`. */
	endKnown: boolean;
	allDay: boolean;
}

/** A multi-day event shows on at most this many days. */
export const MAX_EVENT_DAYS = 60;

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS = [
	"Jan",
	"Feb",
	"Mar",
	"Apr",
	"May",
	"Jun",
	"Jul",
	"Aug",
	"Sep",
	"Oct",
	"Nov",
	"Dec",
];
const MAX_OFFSET_MINUTES = 14 * 60;
const DATE_TIME =
	/^(\d{4})-(\d{2})-(\d{2})(?:[Tt ](\d{2}):(\d{2})(?::(\d{2})(?:[.,]\d+)?)?\s*([Zz]|[+-]\d{2}(?::?\d{2})?)?)?$/;

/**
 * Reads the stored forms (`2026-12-05`, `2026-12-05T19:00`, `2026-12-05T19:00+01:00`) and
 * the ISO 8601 variants pages write, or `null` for anything else.
 */
export function parseEventTime(value: unknown): EventTime | null {
	if (typeof value !== "string") return null;
	const text = dropZoneName(value.trim());
	const match = DATE_TIME.exec(text);
	if (!match) return null;
	const [, y, mo, d, h, mi, s, zone] = match;
	const year = Number(y);
	const month = Number(mo);
	const day = Number(d);
	const validDate =
		year >= 1000 && month >= 1 && month <= 12 && day >= 1 && day <= daysIn(year, month);
	if (!validDate) return null;
	if (h === undefined) return { year, month, day };
	const hour = Number(h);
	const minute = Number(mi);
	if (hour > 23 || minute > 59 || (s !== undefined && Number(s) > 60)) return null;
	const offsetMinutes = zone === undefined ? undefined : parseOffset(zone);
	if (offsetMinutes === null) return null;
	return { year, month, day, time: { hour, minute, offsetMinutes } };
}

/**
 * Drops a trailing RFC 9557 zone name, as in `2026-12-05T19:00+01:00[Europe/Warsaw]`.
 * A regex for this is quadratic on input like `[[[[`, so it scans instead.
 */
function dropZoneName(text: string): string {
	if (!text.endsWith("]")) return text;
	const open = text.indexOf("[", text.lastIndexOf("]", text.length - 2) + 1);
	return open === -1 ? text : text.slice(0, open);
}

/**
 * `Sat 5 Dec 2026, 19:00`, or `Sat 5 Dec 2026` without a time. A time with an offset shows
 * in the viewer's local time, as the calendar places it; a floating one as written.
 */
export function formatEventTime(time: EventTime): string {
	if (time.time?.offsetMinutes !== undefined) {
		const local = localStart(time);
		return `${formatDate(local)}, ${pad(local.getHours())}:${pad(local.getMinutes())}`;
	}
	const date = formatDate(new Date(time.year, time.month - 1, time.day));
	if (!time.time) return date;
	return `${date}, ${pad(time.time.hour)}:${pad(time.time.minute)}`;
}

function formatDate(day: Date): string {
	return `${WEEKDAYS[day.getDay()]} ${day.getDate()} ${MONTHS[day.getMonth()]} ${day.getFullYear()}`;
}

/**
 * The event's span in local time, or `null` when `starts` is missing or unreadable: the
 * event's date isn't set. Without an end, an event lasts until its start's day ends.
 */
export function eventSpan(fields: Record<string, unknown>): EventSpan | null {
	const starts = parseEventTime(fields.starts);
	if (!starts) return null;
	const start = localStart(starts);
	const ends = parseEventTime(fields.ends);
	const end = ends ? localEnd(ends) : null;
	// A date-only end is the midnight after its day, so it must pass the start to cover
	// the start's day.
	const endKnown = end !== null && (ends?.time ? end >= start : end > start);
	return {
		start,
		end: endKnown ? end : nextDay(startOfDay(start)),
		endKnown,
		allDay: !starts.time,
	};
}

/** Not yet ended, which includes happening now. */
export function isUpcoming(span: EventSpan, now: Date): boolean {
	return span.end > now;
}

export function isHappening(span: EventSpan, now: Date): boolean {
	return span.start <= now && now < span.end;
}

/**
 * The local days the event spans, each as its local midnight, at most `MAX_EVENT_DAYS`.
 * Always its start's day, even when it ends the moment it starts.
 */
export function eventDays(span: EventSpan): Date[] {
	const days = [startOfDay(span.start)];
	for (
		let day = nextDay(days[0]);
		day < span.end && days.length < MAX_EVENT_DAYS;
		day = nextDay(day)
	) {
		days.push(day);
	}
	return days;
}

/**
 * Items on each local day they span, keyed by `dayKey`. Within a day, all-day events come
 * first, then the rest by start, so one running on from an earlier day follows them.
 */
export function eventsByDay<T>(
	items: T[],
	spanOf: (item: T) => EventSpan | null,
): Map<string, T[]> {
	const buckets = new Map<string, { item: T; span: EventSpan }[]>();
	for (const item of items) {
		const span = spanOf(item);
		if (!span) continue;
		for (const day of eventDays(span)) {
			const key = dayKey(day);
			const bucket = buckets.get(key);
			if (bucket) bucket.push({ item, span });
			else buckets.set(key, [{ item, span }]);
		}
	}
	const days = new Map<string, T[]>();
	for (const key of [...buckets.keys()].sort()) {
		const bucket = buckets.get(key) ?? [];
		bucket.sort(
			(a, b) =>
				Number(b.span.allDay) - Number(a.span.allDay) ||
				a.span.start.getTime() - b.span.start.getTime(),
		);
		days.set(key, bucket.map(({ item }) => item));
	}
	return days;
}

/** The local date as `YYYY-MM-DD`. */
export function dayKey(date: Date): string {
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/**
 * The 42 local days (6 weeks) a month grid shows for `month` (1 to 12). `weekStart` is the
 * first column's weekday, 0 for Sunday to 6 for Saturday, as `Date.getDay` counts.
 */
export function monthGridDays(year: number, month: number, weekStart: number): Date[] {
	const first = new Date(year, month - 1, 1);
	const lead = (first.getDay() - weekStart + 7) % 7;
	return Array.from(
		{ length: 42 },
		(_, i) => new Date(year, month - 1, 1 - lead + i),
	);
}

function localStart(time: EventTime): Date {
	const { year, month, day } = time;
	if (!time.time) return new Date(year, month - 1, day);
	const { hour, minute, offsetMinutes } = time.time;
	if (offsetMinutes === undefined) return new Date(year, month - 1, day, hour, minute);
	return new Date(Date.UTC(year, month - 1, day, hour, minute) - offsetMinutes * 60_000);
}

/** A date-only end means through that day. */
function localEnd(time: EventTime): Date {
	const start = localStart(time);
	return time.time ? start : nextDay(start);
}

function startOfDay(date: Date): Date {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

function nextDay(day: Date): Date {
	return new Date(day.getFullYear(), day.getMonth(), day.getDate() + 1);
}

/** `null` for an offset that can't be real. */
function parseOffset(zone: string): number | null {
	if (zone === "Z" || zone === "z") return 0;
	const digits = zone.slice(1).replace(":", "");
	const hours = Number(digits.slice(0, 2));
	const minutes = digits.length === 4 ? Number(digits.slice(2)) : 0;
	const total = hours * 60 + minutes;
	if (minutes > 59 || total > MAX_OFFSET_MINUTES) return null;
	return zone.startsWith("-") ? -total : total;
}

function daysIn(year: number, month: number): number {
	const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
	if (month === 2) return leap ? 29 : 28;
	return [4, 6, 9, 11].includes(month) ? 30 : 31;
}

function pad(n: number): string {
	return String(n).padStart(2, "0");
}
