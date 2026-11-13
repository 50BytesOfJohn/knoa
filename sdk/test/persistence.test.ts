import { afterEach, expect, it, vi } from "vitest";
import { HttpError } from "../src/api";
import { FakeServer } from "../src/fake-server";
import { createWorkspace, updateWorkspace } from "../src/mutations";
import { Replica } from "../src/replica";
import { MemoryStorage, type Storage } from "../src/storage";
import { StorageWriter } from "../src/storage-writer";
import { clientFor, started } from "./fixtures";

afterEach(() => {
	vi.useRealTimers();
	vi.restoreAllMocks();
});

function deferred() {
	let resolve!: () => void;
	const promise = new Promise<void>((done) => {
		resolve = done;
	});
	return { promise, resolve };
}

it("does not restore an acknowledged enqueue after a delayed save failure", async () => {
	const server = new FakeServer();
	const storage = new MemoryStorage();
	const client = await started(clientFor(server, storage, null));
	const entered = deferred();
	const release = deferred();
	const acknowledged = deferred();
	const liveAcknowledged = deferred();
	const flushed = deferred();
	const save = storage.save.bind(storage);
	let flushing = false;
	const push = client.api.push.bind(client.api);
	vi.spyOn(client.api, "push").mockImplementationOnce(async (request) => {
		let first = true;
		storage.save = async (change) => {
			if (first) {
				first = false;
				entered.resolve();
				await release.promise;
				throw new Error("quota exceeded");
			}
			await save(change);
			if (change.dequeue.length > 0) acknowledged.resolve();
			if (flushing) flushed.resolve();
		};
		client.mutate([createWorkspace("Work", 1).mutation]);
		await entered.promise;
		return push(request);
	});
	const unsubscribe = client.subscribe(() => {
		if (client.getSnapshot().pending === 0) liveAcknowledged.resolve();
	});
	try {
		client.mutate([createWorkspace("Personal", 0).mutation]);
		await entered.promise;
		await liveAcknowledged.promise;
		expect(client.getSnapshot().pending).toBe(0);
		release.resolve();
		await acknowledged.promise;
		flushing = true;
		client.sync();
		await client.settled();
		await flushed.promise;
		expect((await storage.load())?.outbox).toEqual([]);
	} finally {
		unsubscribe();
		client.stop();
	}
});

it("keeps deletion and revival in order across a failed save", async () => {
	const server = new FakeServer();
	const { id, mutation } = createWorkspace("Personal", 0);
	server.push([mutation]);
	const replica = new Replica();
	const storage = new MemoryStorage();
	const writer = new StorageWriter(replica, storage);
	const pull = () => {
		const page = server.pull(replica.cursor);
		replica.applyPull(page.changes, page.cursor);
		writer.save();
	};
	pull();
	await writer.settled();

	const entered = deferred();
	const release = deferred();
	const save = storage.save.bind(storage);
	storage.save = async () => {
		storage.save = save;
		entered.resolve();
		await release.promise;
		throw new Error("quota exceeded");
	};
	server.push([{ ...mutation, mid: "delete", op: "delete" }]);
	pull();
	await entered.promise;
	server.push([{ ...mutation, mid: "revive" }]);
	pull();
	release.resolve();
	await writer.settled();
	expect((await storage.load())?.rows.map((row) => row.key)).toEqual([id]);
	expect((await storage.load())?.cursor).toBe(3);

	server.push([{ ...mutation, mid: "delete-again", op: "delete" }]);
	pull();
	await writer.settled();
	expect((await storage.load())?.rows).toEqual([]);
	expect((await storage.load())?.cursor).toBe(4);
});

it.each(["update", "delete", "revive"])("reloads a delayed stale-tab write after a newer %s", async (operation) => {
	const server = new FakeServer();
	const { id, mutation } = createWorkspace("Old", 0);
	server.push([mutation]);
	const database = new MemoryStorage();
	if (operation === "revive") {
		const seed = await started(clientFor(server, database, null));
		seed.stop();
		server.push([{ ...mutation, mid: "delete", op: "delete" }]);
	}
	const { storage, entered, release, completed } = delayFirstSave(database);
	const old = clientFor(server, storage, null);
	await old.start();
	await entered.promise;
	server.push([operation === "delete"
		? { ...mutation, mid: "delete", op: "delete" }
		: { ...mutation, entity: "workspace", mid: "new", op: "put", data: { id, name: "New" } },
	]);
	const newer = await started(clientFor(server, database, null));
	try {
		release.resolve();
		await completed.promise;
		await old.settled();
		newer.sync();
		await newer.settled();
		const stored = await database.load();
		expect(stored?.cursor).toBe(operation === "revive" ? 3 : 2);
		const row = stored?.rows.find((row) => row.key === id);
		if (operation === "delete") expect(row).toBeUndefined();
		else expect(row?.data).toMatchObject({ name: "New" });
		const expected = operation === "delete" ? undefined : "New";
		expect(newer.getSnapshot().library.workspace.get(id)?.name).toBe(expected);
		expect(old.getSnapshot().library.workspace.get(id)?.name).toBe(expected);
	} finally {
		old.stop();
		newer.stop();
	}
});

it("keeps a local rename visible while resolving another tab's save", async () => {
	const server = new FakeServer();
	const { id, mutation } = createWorkspace("Old", 0);
	server.push([mutation]);
	const storage = new MemoryStorage();
	const first = await started(clientFor(server, storage, null));
	const second = await started(clientFor(server, storage, null));
	first.sync();
	await first.settled();
	const names: (string | undefined)[] = [];
	const unsubscribe = second.subscribe(() => {
		names.push(second.getSnapshot().library.workspace.get(id)?.name);
	});
	try {
		second.mutate([updateWorkspace(id, { name: "Renamed" })]);
		await second.settled();
		expect(names.length).toBeGreaterThan(0);
		expect(new Set(names)).toEqual(new Set(["Renamed"]));
		expect(second.getSnapshot().pending).toBe(0);
		expect((await storage.load())?.rows.find((row) => row.key === id)?.data)
			.toMatchObject({ name: "Renamed" });
	} finally {
		unsubscribe();
		first.stop();
		second.stop();
	}
});

it("keeps both tabs' unpushed edits, including an edit made during conflict reload", async () => {
	vi.useFakeTimers();
	const server = new FakeServer();
	const storage = new MemoryStorage();
	const first = await started(clientFor(server, storage, null));
	const second = await started(clientFor(server, storage, null));
	vi.spyOn(first.api, "push").mockRejectedValue(new TypeError("offline"));
	const push = vi.spyOn(second.api, "push").mockRejectedValue(new TypeError("offline"));
	const a = createWorkspace("A", 0).mutation;
	const b = createWorkspace("B", 1).mutation;
	const c = createWorkspace("C", 2).mutation;
	first.mutate([a]);
	await first.settled();

	const reloading = deferred();
	const release = deferred();
	const load = storage.load.bind(storage);
	storage.load = async () => {
		storage.load = load;
		const state = await load();
		reloading.resolve();
		await release.promise;
		return state;
	};
	try {
		second.mutate([b]);
		await reloading.promise;
		second.mutate([c]);
		release.resolve();
		await second.settled();
		expect((await storage.load())?.outbox.map((m) => m.mid)).toEqual([a.mid, b.mid, c.mid]);
		expect(second.getSnapshot().pending).toBe(3);

		const acknowledger = await started(clientFor(server, storage, null));
		acknowledger.stop();
		push.mockRejectedValue(new HttpError(503, "unavailable", "Try later"));
		second.sync();
		await second.settled();
		expect((await storage.load())?.outbox).toEqual([]);
		expect(second.getSnapshot().pending).toBe(0);
	} finally {
		first.stop();
		second.stop();
	}
});

function delayFirstSave(database: Storage) {
	const entered = deferred();
	const release = deferred();
	const completed = deferred();
	let first = true;
	const storage: Storage = {
		load: () => database.load(),
		clear: () => database.clear(),
		save: async (change) => {
			if (first) {
				first = false;
				entered.resolve();
				await release.promise;
				try {
					await database.save(change);
				} finally {
					completed.resolve();
				}
			} else {
				await database.save(change);
			}
		},
	};
	return { storage, entered, release, completed };
}
