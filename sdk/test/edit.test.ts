import { describe, expect, it } from "vitest";
import { addLink, editBookmark } from "../src/mutations";
import type { Bookmark } from "../src/protocol";
import { Replica } from "../src/replica";

const WS = "6f1c2a7e-9a51-4b8e-a3b4-1f9e2d6c8a10";

function organized(): Bookmark {
	const added = addLink(new Replica().library(), WS, "example.com/a");
	const mutation = added?.mutations[0];
	if (mutation?.op !== "put") throw new Error("expected a put");
	return { ...(mutation.data as Bookmark), summary: "The AI's summary" };
}

describe("editBookmark", () => {
	it("leaves the AI's summary to the AI when only the title changes", () => {
		const bookmark = organized();
		const edit = editBookmark(bookmark, {
			title: " My title ",
			summary: "The AI's summary",
		});
		expect(edit?.op === "put" && edit.data).toEqual({ title: "My title" });
	});
});
