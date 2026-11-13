import { describe, expect, it } from "vitest";
import { Backoff } from "../src/backoff";

describe("Backoff", () => {
	it("doubles up to the max, each wait between half and all of it", () => {
		const shortest = new Backoff(1_000, 4_000, () => 1);
		const longest = new Backoff(1_000, 4_000, () => 0);
		expect([1, 2, 3, 4].map(() => shortest.next())).toEqual([
			500, 1_000, 2_000, 2_000,
		]);
		expect([1, 2, 3, 4].map(() => longest.next())).toEqual([
			1_000, 2_000, 4_000, 4_000,
		]);
	});
});
