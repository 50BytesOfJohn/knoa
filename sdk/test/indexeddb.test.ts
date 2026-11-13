import { IDBFactory } from "fake-indexeddb";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import revive from "../fixtures/tombstone-revive.json";
import { IndexedDbStorage } from "../src/indexeddb";
import { createWorkspace } from "../src/mutations";
import type { PullResponse } from "../src/protocol";
import { rowId, StorageConflictError, type StorageChange, type StoredState } from "../src/storage";
import { workspaceChange } from "./fixtures";

beforeEach(() => vi.stubGlobal("indexedDB", new IDBFactory()));
afterEach(() => vi.unstubAllGlobals());

it.each([1, 2])("upgrades version %i without losing rows, cursor or unsent edits", async (version) => {
	const page = revive.steps[1]?.expect as unknown as PullResponse;
	const rows = page.changes.flatMap((change) => change.deleted ? [] : [{
		entity: change.entity, key: change.key, data: change.data,
	}]);
	const outbox = [createWorkspace("A", 0).mutation, createWorkspace("B", 1).mutation];
	const original = { cursor: page.cursor, rows, outbox };
	await legacyDatabase("upgrade", version, original);
	const storage = new IndexedDbStorage("upgrade");
	const upgraded = await storage.load();
	expect(upgraded).toEqual({ ...original, rows: expect.arrayContaining(rows), revision: 0 });
	expect(upgraded?.rows).toHaveLength(rows.length);
	await storage.save({
		revision: upgraded?.revision, cursor: page.cursor,
		put: [], remove: [], enqueue: [], dequeue: [],
	});
	expect(await storage.load()).toEqual({ ...upgraded, revision: 1 });
});

it("compares revisions and commits rows, cursor and outbox in one transaction", async () => {
	const first = new IndexedDbStorage("shared");
	const second = new IndexedDbStorage("shared");
	const a = workspaceChange("a", 1);
	const b = workspaceChange("b", 1);
	if (a.deleted || b.deleted) throw new Error("expected rows");
	const oldRow = { entity: a.entity, key: a.key, data: a.data };
	const otherRow = { entity: b.entity, key: b.key, data: b.data };
	const queued = createWorkspace("Pending", 0).mutation;
	await first.save({ cursor: 1, put: [oldRow, otherRow], remove: [], enqueue: [queued], dequeue: [] });
	const [left, right] = await Promise.all([first.load(), second.load()]);
	const winner = createWorkspace("Winner", 0).mutation;
	const loser = createWorkspace("Loser", 0).mutation;
	const newRow = { ...oldRow, data: { ...a.data, name: "New" } };
	const fresh: StorageChange = {
		revision: left?.revision, cursor: 2, put: [newRow], remove: [],
		enqueue: [winner], dequeue: [queued.mid],
	};
	const stale: StorageChange = {
		revision: right?.revision, cursor: 1, put: [oldRow], remove: [otherRow],
		enqueue: [loser], dequeue: [winner.mid],
	};
	const results = await Promise.allSettled([first.save(fresh), second.save(stale)]);
	expect(results).toEqual([
		{ status: "fulfilled", value: undefined },
		{ status: "rejected", reason: expect.any(StorageConflictError) },
	]);
	const committed = await second.load();
	expect(committed).toEqual({
		revision: 2, cursor: 2, rows: [newRow, otherRow], outbox: [winner],
	});

	const uncloneable = { ...loser, uncloneable: () => {} };
	await expect(second.save({
		...stale, revision: committed?.revision, enqueue: [uncloneable],
	})).rejects.toMatchObject({ name: "DataCloneError" });
	expect(await first.load()).toEqual(committed);
});

function legacyDatabase(name: string, version: number, state: StoredState): Promise<void> {
	return new Promise((resolve, reject) => {
		const open = indexedDB.open(name, version);
		open.onupgradeneeded = () => {
			const rows = open.result.createObjectStore("rows");
			for (const row of state.rows) rows.put(row, rowId(row));
			const meta = open.result.createObjectStore("meta");
			if (version === 1) {
				meta.put({ cursor: state.cursor, outbox: state.outbox }, "state");
			} else {
				meta.put({ cursor: state.cursor }, "state");
				const outbox = open.result.createObjectStore("outbox");
				[...state.outbox].reverse().forEach((mutation, index) => {
					outbox.put({ seq: state.outbox.length - index, mutation }, mutation.mid);
				});
			}
		};
		open.onsuccess = () => {
			open.result.close();
			resolve();
		};
		open.onerror = () => reject(open.error);
	});
}
