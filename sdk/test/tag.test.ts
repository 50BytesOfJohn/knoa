import { expect, it } from "vitest";
import { normalizeTag } from "../src/tag";

it("spells tags like knoa_core::normalize_tag", () => {
	const cases: [string, string][] = [
		["Film Noir ", "film-noir"],
		["film-noir", "film-noir"],
		["Vienna", "vienna"],
		["#Rust", "rust"],
		["##two  words", "two-words"],
		["snake_case_tag", "snake-case-tag"],
		["  -- dashes--and__underscores  ", "dashes-and-underscores"],
		["Ünïcode Tag", "ünïcode-tag"],
		["", ""],
		["#", ""],
	];
	for (const [input, expected] of cases)
		expect(normalizeTag(input)).toBe(expected);
});
