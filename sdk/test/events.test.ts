import { afterAll, beforeAll, expect, it, vi } from "vitest";
import {
	eventSpan,
	eventsByDay,
	isUpcoming,
	monthGridDays,
	parseEventTime,
} from "../src/events";
import { displayField } from "../src/kinds";

// Every local-time expectation below is in New York.
beforeAll(() => {
	vi.stubEnv("TZ", "America/New_York");
});
afterAll(() => {
	vi.unstubAllEnvs();
});

it("reads the forms knoa_core stores and refuses loose text", () => {
	expect(parseEventTime("2026-12-05T19:00:00Z[UTC]")).toEqual({
		year: 2026,
		month: 12,
		day: 5,
		time: { hour: 19, minute: 0, offsetMinutes: 0 },
	});
	expect(parseEventTime("2026-12-05")).toEqual({ year: 2026, month: 12, day: 5 });
	for (const loose of [
		"Dec 5",
		"2026-02-29",
		"2026-12-05T24:00",
		"2026-12-05T19:00+15:00",
		"0099-12-05",
	])
		expect(parseEventTime(loose), loose).toBeNull();
});

it("shows a time with an offset in the viewer's local time, as the calendar does", () => {
	const starts = { name: "starts", type: "datetime" as const, description: "" };

	// 19:00 in Los Angeles is 22:00 in New York; a floating time stays as written.
	expect(displayField(starts, "2026-12-05T19:00-08:00")).toBe("Sat 5 Dec 2026, 22:00");
	expect(displayField(starts, "2026-12-05T19:00")).toBe("Sat 5 Dec 2026, 19:00");
	expect(displayField(starts, "2026-12-05")).toBe("Sat 5 Dec 2026");
});

it("places an instant on the viewer's local day", () => {
	const late = { title: "Paris late show", fields: { starts: "2026-12-06T02:00+01:00" } };
	const floating = { title: "Doors", fields: { starts: "2026-12-06T02:00" } };

	const days = eventsByDay([late, floating], (e) => eventSpan(e.fields));

	// 02:00 in Paris is 20:00 the evening before in New York; a floating time stays put.
	expect(days.get("2026-12-05")?.map((e) => e.title)).toEqual(["Paris late show"]);
	expect(days.get("2026-12-06")?.map((e) => e.title)).toEqual(["Doors"]);
});

it("shows a multi-day event on every day through its end date", () => {
	const festival = { starts: "2026-12-30", ends: "2027-01-02" };

	const days = eventsByDay([festival], eventSpan);

	expect([...days.keys()]).toEqual([
		"2026-12-30",
		"2026-12-31",
		"2027-01-01",
		"2027-01-02",
	]);
});

it("keeps an event upcoming until it ends, or until its day ends without an end", () => {
	const talk = eventSpan({ starts: "2026-12-05T19:00", ends: "2026-12-05T21:00" });
	const meetup = eventSpan({ starts: "2026-12-05T19:00", ends: "2026-12-05T08:00" });
	const evening = new Date(2026, 11, 5, 22, 0);

	expect(talk && isUpcoming(talk, evening)).toBe(false);
	expect(talk?.endKnown).toBe(true);
	// An end before the start is ignored, so the meetup runs to midnight, an end no view
	// should show.
	expect(meetup && isUpcoming(meetup, evening)).toBe(true);
	expect(meetup?.endKnown).toBe(false);
	expect(meetup && isUpcoming(meetup, new Date(2026, 11, 6, 0, 1))).toBe(false);
});

it("ends an event whose end equals its start right then, not at midnight", () => {
	const moment = eventSpan({ starts: "2026-12-05T19:00", ends: "2026-12-05T19:00" });

	expect(moment && isUpcoming(moment, new Date(2026, 11, 5, 20, 0))).toBe(false);
});

it("ignores a date-only end before the start's date", () => {
	const fair = eventSpan({ starts: "2026-12-05", ends: "2026-12-04" });

	expect(fair?.endKnown).toBe(false);
	expect([...eventsByDay([fair], (span) => span).keys()]).toEqual(["2026-12-05"]);
});

it("keeps an event that ends as it starts at midnight on its day", () => {
	const instant = { starts: "2026-12-05T00:00", ends: "2026-12-05T00:00" };

	expect([...eventsByDay([instant], eventSpan).keys()]).toEqual(["2026-12-05"]);
});

it("lists all-day events before one running on from an earlier day", () => {
	const trip = { starts: "2026-12-04T10:00", ends: "2026-12-06T18:00" };
	const fair = { starts: "2026-12-05" };

	expect(eventsByDay([trip, fair], eventSpan).get("2026-12-05")).toEqual([fair, trip]);
});

it("starts the month grid on the given weekday", () => {
	const days = monthGridDays(2026, 12, 1);

	expect(days).toHaveLength(42);
	expect(days[0]).toEqual(new Date(2026, 10, 30));
});
