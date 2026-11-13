import { afterEach, expect, it, vi } from "vitest";
import { SyncClient } from "../src/client";
import { FakeServer } from "../src/fake-server";
import { MemoryStorage, type StoredState } from "../src/storage";

afterEach(() => {
	vi.useRealTimers();
	vi.unstubAllGlobals();
});

function deferred<T>() {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((done) => {
		resolve = done;
	});
	return { promise, resolve };
}

/** A client whose every outside effect is counted. */
function watched(options: {
	load?: () => Promise<StoredState | null>;
	getToken?: () => Promise<string | null>;
}) {
	const server = new FakeServer();
	const storage = new MemoryStorage();
	if (options.load) storage.load = options.load;
	const fetch = vi.fn(server.fetch);
	const socket = vi.fn(server.socket);
	const listen = vi.fn();
	vi.stubGlobal("window", {
		addEventListener: listen,
		removeEventListener: vi.fn(),
	});
	vi.stubGlobal("document", {
		addEventListener: listen,
		removeEventListener: vi.fn(),
	});
	const client = new SyncClient({
		apiUrl: "http://api.test",
		getToken: options.getToken ?? (async () => "token"),
		storage,
		fetch,
		socket,
	});
	return { client, fetch, socket, listen };
}

it("does nothing once stopped while storage was loading", async () => {
	vi.useFakeTimers();
	const load = deferred<StoredState | null>();
	const { client, fetch, socket, listen } = watched({
		load: () => load.promise,
	});
	const starting = client.start();
	client.stop();
	load.resolve(null);
	await starting;
	await client.settled();

	expect(fetch).not.toHaveBeenCalled();
	expect(socket).not.toHaveBeenCalled();
	expect(listen).not.toHaveBeenCalled();
	expect(vi.getTimerCount()).toBe(0);
});

it("sends nothing with a token that arrives after stopping", async () => {
	vi.useFakeTimers();
	const token = deferred<string | null>();
	const { client, fetch, socket } = watched({
		getToken: () => token.promise,
	});
	await client.start();
	client.stop();
	token.resolve("token");
	await client.settled();
	await vi.runAllTimersAsync();

	expect(fetch).not.toHaveBeenCalled();
	expect(socket).not.toHaveBeenCalled();
	expect(vi.getTimerCount()).toBe(0);
});

it("starts once however often start is called", async () => {
	const load = vi.fn(async () => null);
	const { client, socket, listen } = watched({ load });
	await Promise.all([client.start(), client.start()]);
	await client.start();
	await client.settled();

	expect(load).toHaveBeenCalledTimes(1);
	expect(socket).toHaveBeenCalledTimes(1);
	expect(listen).toHaveBeenCalledTimes(4);
	client.stop();
});
