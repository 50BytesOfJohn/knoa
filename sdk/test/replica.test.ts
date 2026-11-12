import type { Bookmark, Change } from "../src/protocol";
import { describe, expect, it } from "vitest";
import { addLink, addTag, removeTag } from "../src/mutations";
import { Replica } from "../src/replica";
import { workspaceChange } from "./fixtures";

const WS = "6f1c2a7e-9a51-4b8e-a3b4-1f9e2d6c8a10";

function bookmarkRow(replica: Replica, url: string): Bookmark {
	const added = addLink(replica.library(), WS, url);
	if (!added) throw new Error("not a link");
	const [mutation] = added.mutations;
	if (mutation?.op !== "put") throw new Error("expected a put");
	return mutation.data as Bookmark;
}

const put = (row: Bookmark, version: number): Change => ({
	entity: "bookmark",
	key: row.id,
	version,
	deleted: false,
	data: row,
});

describe("Replica", () => {
	it("shows children only once their parent arrives", () => {
		const replica = new Replica();
		const row = bookmarkRow(replica, "example.com/a");
		const tag = addTag(row.id, "Reading");
		if (tag?.op !== "put") throw new Error("expected a put");
		replica.applyPull(
			[
				{
					entity: "bookmark_tag",
					key: tag.key,
					version: 1,
					deleted: false,
					data: tag.data,
				} as Change,
				put(row, 2),
			],
			2,
		);
		expect(replica.library().bookmark.size).toBe(0);

		replica.applyPull([workspaceChange(WS, 3)], 3);
		const library = replica.library();
		expect(library.bookmark.get(row.id)?.url).toBe("https://example.com/a");
		expect(library.tags.get(row.id)?.map((t) => t.tag)).toEqual(["reading"]);
	});

	it("deletes a workspace's folders, bookmarks and their links on its tombstone", () => {
		const replica = new Replica();
		const row = bookmarkRow(replica, "example.com/a");
		const tag = addTag(row.id, "x");
		if (tag?.op !== "put") throw new Error("expected a put");
		replica.applyPull(
			[
				workspaceChange(WS, 1),
				put(row, 2),
				{
					entity: "bookmark_tag",
					key: tag.key,
					version: 3,
					deleted: false,
					data: tag.data,
				} as Change,
			],
			3,
		);
		replica.applyPull(
			[{ entity: "workspace", key: WS, version: 4, deleted: true }],
			4,
		);
		replica.applyPull([workspaceChange(WS, 5)], 5);

		const library = replica.library();
		expect(library.workspaces).toHaveLength(1);
		expect(library.bookmark.size).toBe(0);
		expect(library.tags.size).toBe(0);
	});

	it("keeps pending edits over pulled rows, column by column", () => {
		const replica = new Replica();
		const row = bookmarkRow(replica, "example.com/a");
		replica.applyPull([workspaceChange(WS, 1), put(row, 2)], 2);
		replica.enqueue([
			{
				mid: "m1",
				entity: "bookmark",
				key: row.id,
				actor: "user",
				op: "put",
				data: { up_next: true },
			},
		]);

		replica.applyPull(
			[
				put(
					{
						...row,
						up_next: false,
						title: "From the server",
						summary: "Organized",
					},
					3,
				),
			],
			3,
		);
		const seen = replica.library().bookmark.get(row.id);
		expect(seen?.up_next).toBe(true);
		expect(seen?.title).toBe("From the server");
		expect(seen?.summary).toBe("Organized");
	});

	it("rolls a change back when the server doesn't take it", () => {
		const replica = new Replica();
		const row = bookmarkRow(replica, "example.com/a");
		replica.applyPull([workspaceChange(WS, 1), put(row, 2)], 2);
		const remove = removeTag(row.id, "x");
		replica.enqueue([addTag(row.id, "x") ?? remove, remove]);
		expect(replica.library().tags.get(row.id)?.[0]?.excluded).toBe(true);

		const rejections = replica.settle(
			replica.pending.map((m) => ({
				mid: m.mid,
				status: "rejected" as const,
				reason: "invalid" as const,
			})),
		);
		expect(rejections).toHaveLength(2);
		expect(replica.library().tags.get(row.id)).toBeUndefined();
	});
});
