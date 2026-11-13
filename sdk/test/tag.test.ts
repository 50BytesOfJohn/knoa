import { expect, it } from "vitest";
import fixture from "../fixtures/tag-keys.json";
import { addTag } from "../src/mutations";

// The same cases run against `knoa_core::normalize_tag` (crates/knoa-core/tests/tag_keys.rs),
// whose spelling is canonical.
it("keys tags like knoa_core", () => {
	for (const { input, key } of fixture.cases)
		expect(addTag(fixture.bookmark, input)?.key ?? null, input).toBe(key);
});
