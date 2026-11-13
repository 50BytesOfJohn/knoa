import { expect, it } from "vitest";
import { FakeServer } from "../src/fake-server";
import { createWorkspace } from "../src/mutations";
import { Replica } from "../src/replica";
import { MemoryStorage } from "../src/storage";
import { StorageWriter } from "../src/storage-writer";
import { clientFor, started } from "./fixtures";

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
	const flushed = deferred();
	const save = storage.save.bind(storage);
	let first = true;
	let flushing = false;
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
	try {
		client.mutate([createWorkspace("Personal", 0).mutation]);
		await entered.promise;
		await client.settled();
		expect(client.getSnapshot().pending).toBe(0);
		release.resolve();
		await acknowledged.promise;
		flushing = true;
		client.sync();
		await client.settled();
		await flushed.promise;
		expect((await storage.load())?.outbox).toEqual([]);
	} finally {
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
