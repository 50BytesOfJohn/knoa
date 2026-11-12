import { MAX_PUSH, MAX_PUSH_BYTES, type Mutation } from "../src/protocol";
import { afterEach, describe, expect, it, vi } from "vitest";
import { nextBatch } from "../src/batch";
import { FakeServer } from "../src/fake-server";
import { addLink, addTag, createWorkspace, setUpNext } from "../src/mutations";
import { bookmarksIn } from "../src/select";
import { MemoryStorage } from "../src/storage";
import { clientFor, started } from "./fixtures";

afterEach(() => {
	vi.useRealTimers();
});

function links(
	client: ReturnType<typeof clientFor>,
	workspace: string,
	count: number,
): Mutation[] {
	const library = client.getSnapshot().library;
	return Array.from({ length: count }, (_, i) => {
		const added = addLink(library, workspace, `example.com/${i}`);
		if (!added) throw new Error("not a link");
		return added.mutations;
	}).flat();
}

describe("SyncClient", () => {
	it("pushes the outbox in order, in batches of at most MAX_PUSH", async () => {
		const server = new FakeServer();
		const client = await started(clientFor(server));
		const { id, mutation } = createWorkspace("Personal", 0);
		client.mutate([mutation, ...links(client, id, MAX_PUSH + 20)]);
		await client.settled();

		expect(server.pushes.map((batch) => batch.length)).toEqual([MAX_PUSH, 21]);
		const sent = server.pushes.flat().map((m) => m.mid);
		expect(client.getSnapshot().pending).toBe(0);
		expect(new Set(sent).size).toBe(sent.length);
		expect(server.pushes[0]?.[0]?.entity).toBe("workspace");
		expect(server.tables.bookmark.size).toBe(MAX_PUSH + 20);
		client.stop();
	});

	it("retries network and server errors with the same mids", async () => {
		vi.useFakeTimers();
		const server = new FakeServer();
		const client = await started(clientFor(server));
		server.failNext("network", 503);
		const { mutation } = createWorkspace("Personal", 0);
		client.mutate([mutation]);
		await client.settled();
		expect(client.getSnapshot().status).toBe("offline");
		expect(client.getSnapshot().pending).toBe(1);

		await vi.advanceTimersByTimeAsync(1_000);
		await client.settled();
		expect(client.getSnapshot().status).toBe("error");

		await vi.advanceTimersByTimeAsync(2_000);
		await client.settled();
		expect(client.getSnapshot().status).toBe("synced");
		expect(client.getSnapshot().pending).toBe(0);
		expect(server.pushes.flat().map((m) => m.mid)).toEqual([mutation.mid]);
		expect(server.tables.workspace.size).toBe(1);
		client.stop();
	});

	it("still pulls while the server keeps failing a push", async () => {
		vi.useFakeTimers();
		const server = new FakeServer();
		const web = await started(clientFor(server, new MemoryStorage(), null));
		const mac = await started(clientFor(server));
		mac.mutate([createWorkspace("Mac", 0).mutation]);
		await mac.settled();

		server.failNext(500);
		web.mutate([createWorkspace("Web", 1).mutation]);
		await web.settled();
		const snapshot = web.getSnapshot();
		expect(snapshot.status).toBe("error");
		expect(snapshot.pending).toBe(1);
		expect(snapshot.library.workspaces.map((w) => w.name)).toEqual([
			"Mac",
			"Web",
		]);
		mac.stop();
		web.stop();
	});

	it("shows a change at once and rolls it back when the quota rejects it", async () => {
		const server = new FakeServer({ bookmarkLimit: 1 });
		const client = await started(clientFor(server));
		const { id, mutation } = createWorkspace("Personal", 0);
		client.mutate([mutation, ...links(client, id, 1)]);
		await client.settled();

		client.mutate(links(client, id, 2));
		const everything = { type: "everything" } as const;
		expect(
			bookmarksIn(client.getSnapshot().library, id, everything),
		).toHaveLength(2);

		await client.settled();
		const snapshot = client.getSnapshot();
		expect(bookmarksIn(snapshot.library, id, everything)).toHaveLength(1);
		expect(snapshot.error?.kind).toBe("quota");
		client.stop();
	});

	it("pulls another device's change when poked", async () => {
		const server = new FakeServer();
		const mac = await started(clientFor(server));
		const { id, mutation } = createWorkspace("Personal", 0);
		const added = addLink(mac.getSnapshot().library, id, "example.com/a");
		if (!added) throw new Error("not a link");
		mac.mutate([mutation, ...added.mutations]);
		await mac.settled();

		const web = await started(clientFor(server));
		server.failNext(503);
		web.mutate([setUpNext(added.id, true)]);
		await web.settled();

		mac.mutate([
			addTag(added.id, "Later") as Mutation,
			setUpNext(added.id, false),
		]);
		await mac.settled();
		await vi.waitFor(() => {
			expect(web.getSnapshot().library.tags.get(added.id)?.[0]?.tag).toBe(
				"later",
			);
		});
		expect(web.getSnapshot().library.bookmark.get(added.id)?.up_next).toBe(
			true,
		);
		mac.stop();
		web.stop();
	});

	it("renders from storage before the network answers", async () => {
		const server = new FakeServer();
		const storage = new MemoryStorage();
		const first = await started(clientFor(server, storage));
		const { id, mutation } = createWorkspace("Personal", 0);
		first.mutate([mutation, ...links(first, id, 3)]);
		await first.settled();
		first.stop();

		server.failNext("network");
		const second = clientFor(server, storage);
		await second.start();
		const snapshot = second.getSnapshot();
		expect(snapshot.ready).toBe(true);
		expect(
			bookmarksIn(snapshot.library, id, { type: "everything" }),
		).toHaveLength(3);
		second.stop();
	});
});

describe("nextBatch", () => {
	it("keeps a push under MAX_PUSH_BYTES", () => {
		const text = "x".repeat(300_000);
		const pending = Array.from(
			{ length: 30 },
			(_, i) =>
				({
					mid: `m${i}`,
					entity: "bookmark_text",
					key: `k${i}`,
					actor: "user",
					op: "put",
					data: { text },
				}) as Mutation,
		);
		const batch = nextBatch(pending);
		expect(batch.length).toBeGreaterThan(0);
		expect(JSON.stringify({ mutations: batch }).length).toBeLessThan(
			MAX_PUSH_BYTES,
		);
		expect(batch.length).toBeLessThan(pending.length);
	});
});

describe("storage", () => {
	it("writes rows again after a failed save", async () => {
		const server = new FakeServer();
		const mac = await started(clientFor(server));
		const { id, mutation } = createWorkspace("Personal", 0);
		mac.mutate([mutation, ...links(mac, id, 2)]);
		await mac.settled();

		let failures = 1;
		const storage = new MemoryStorage();
		const save = storage.save.bind(storage);
		storage.save = async (change) => {
			if (failures-- > 0) throw new Error("quota exceeded");
			return save(change);
		};
		const web = await started(clientFor(server, storage, null));
		web.mutate([
			setUpNext(
				[...web.getSnapshot().library.bookmark.keys()][0] as string,
				true,
			),
		]);
		await web.settled();

		const stored = await storage.load();
		expect(
			stored?.rows.filter((row) => row.entity === "bookmark"),
		).toHaveLength(2);
		mac.stop();
		web.stop();
	});

	it("keeps both tabs' unpushed edits in a shared database", async () => {
		vi.useFakeTimers();
		const server = new FakeServer();
		const storage = new MemoryStorage();
		const first = await started(clientFor(server, storage, null));
		const second = await started(clientFor(server, storage, null));
		server.failNext("network", "network");
		const a = createWorkspace("A", 0).mutation;
		const b = createWorkspace("B", 1).mutation;
		first.mutate([a]);
		await first.settled();
		second.mutate([b]);
		await second.settled();

		const stored = await storage.load();
		expect(stored?.outbox.map((m) => m.mid)).toEqual([a.mid, b.mid]);
		first.stop();
		second.stop();
	});

	it("sends nothing once stopped", async () => {
		const server = new FakeServer();
		const client = await started(clientFor(server, new MemoryStorage(), null));
		client.stop();
		client.mutate([createWorkspace("Late", 0).mutation]);
		client.sync();
		await client.settled();
		expect(server.pushes).toHaveLength(0);
	});
});
