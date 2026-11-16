import { describe, expect, it } from "vitest";
import nested from "../fixtures/nested-folders.json";
import { FakeServer } from "../src/fake-server";
import { parseFolderIcon } from "../src/folder-icon";
import { addLink, addToFolder, createFolder, deleteFolder } from "../src/mutations";
import type { Folder, Mutation, PullResponse } from "../src/protocol";
import { Replica } from "../src/replica";
import { bookmarksIn, FolderTree, folderTree, placeCounts } from "../src/select";

const WS = "11111111-1111-4111-8111-111111111111";
const P = "22222222-2222-4222-8222-222222222222";
const C = "33333333-3333-4333-8333-333333333333";
const G = "44444444-4444-4444-8444-444444444444";

function folder(id: string, parent: string | null, position = 0): Folder {
	return {
		id,
		workspace_id: WS,
		parent_id: parent,
		name: id,
		description: "",
		auto_assign: true,
		color: 0,
		hue: null,
		icon: null,
		position,
	};
}

describe("nested folders", () => {
	it("match the shared fixture, server and replica alike", () => {
		const server = new FakeServer();
		const replica = new Replica();
		for (const step of nested.steps) {
			if (step.push) {
				const mutations = step.push.mutations as Mutation[];
				expect(server.push(mutations)).toEqual(step.expect);
			} else if (step.pull) {
				const { cursor, limit } = step.pull;
				const page = server.pull(cursor, limit);
				expect(page).toEqual(step.expect);
				replica.applyPull((page as PullResponse).changes, page.cursor);
			}
		}
		const library = replica.library();
		expect([...library.folder.keys()].sort()).toEqual(nested.replica.folder);
		const tree = folderTree(library, WS);
		const parents = Object.fromEntries(
			tree.nodes.map((node) => [node.folder.id, node.parent]),
		);
		expect(parents).toEqual(nested.parents);
	});

	it("fall back to the top level when a parent is missing, cyclic or too deep", () => {
		const tree = new FolderTree([
			folder("orphan", "gone"),
			folder("a", "b"),
			folder("b", "a"),
			folder("1", null, 1),
			folder("2", "1"),
			folder("3", "2"),
			folder("4", "3"),
		]);

		expect(tree.nodes).toHaveLength(7);
		for (const id of ["orphan", "1", "4"]) expect(tree.parent(id)).toBeUndefined();
		expect(tree.pathName("3")).toBe("1 › 2 › 3");
		expect(tree.checkMove("1", "3")).toBe("into_itself");
		expect(tree.checkNewChild("3")).toBe("too_deep");
	});

	it("count and show a bookmark once under every parent, and delete deepest first", () => {
		const replica = new Replica();
		replica.applyPull(nested.steps[1]?.expect.changes as PullResponse["changes"], 5);
		const [parent, child, grandchild] = [P, C, G];
		const added = addLink(replica.library(), WS, "example.com/a");
		if (!added) throw new Error("not a link");
		replica.enqueue([
			...added.mutations,
			addToFolder(added.id, child),
			addToFolder(added.id, grandchild),
		]);
		const library = replica.library();

		const counts = placeCounts(library, WS).folders;
		expect([parent, child, grandchild].map((id) => counts.get(id))).toEqual([1, 1, 1]);
		const shown = bookmarksIn(library, WS, { type: "folder", id: parent });
		expect(shown.map((b) => b.id)).toEqual([added.id]);
		const deletes = deleteFolder(library, parent).map((m) => m.key);
		expect(deletes).toEqual([grandchild, child, parent]);
		const { mutation } = createFolder(WS, "Top", 1, 0);
		expect(mutation.op === "put" && "parent_id" in mutation.data).toBe(false);
	});

	it("read icons as glyphs, single emoji or nothing", () => {
		expect(parseFolderIcon("read")).toEqual({ type: "glyph", name: "read" });
		for (const emoji of ["🎨", "❤️", "👩🏽‍💻", "🇵🇱", "1️⃣"]) {
			expect(parseFolderIcon(emoji)).toEqual({ type: "emoji", emoji });
		}
		for (const other of ["rocket", "A", "🎨🎨", " 🎨", "→", null]) {
			expect(parseFolderIcon(other)).toBeNull();
		}
	});
});
