//! An in-memory stand-in for the API Worker, with just enough of the protocol for the
//! client's tests and for working on the web app without the real API. Not for production.

import {
	type Account,
	type Change,
	type Entity,
	type Key,
	MAX_PULL,
	type Mutation,
	type MutationResult,
	type PullResponse,
	type PushResponse,
	type Rows,
} from "./protocol";
import type { SocketFactory, SocketLike } from "./socket";
import { deleteRow, emptyTables, putRow, type Tables } from "./tables";

interface Versioned {
	entity: Entity;
	key: Key;
	version: number;
	deleted: boolean;
}

export interface FakeServerState {
	version: number;
	rows: { entity: Entity; key: Key; version: number; data: Rows[Entity] }[];
	tombstones: Versioned[];
	results: [string, MutationResult][];
}

export interface FakeServerOptions {
	bookmarkLimit?: number;
	plan?: Account["plan"];
	state?: FakeServerState;
}

export class FakeServer {
	readonly tables: Tables = emptyTables();
	private versions = new Map<string, Versioned>();
	private version = 0;
	private results = new Map<string, MutationResult>();
	private sockets = new Set<FakeSocket>();
	private failures: (number | "network")[] = [];
	/** Every push request, for tests to look at. */
	readonly pushes: Mutation[][] = [];
	bookmarkLimit: number;
	plan: Account["plan"];

	constructor(options: FakeServerOptions = {}) {
		this.bookmarkLimit = options.bookmarkLimit ?? 10_000;
		this.plan = options.plan ?? "free";
		if (options.state) this.restore(options.state);
	}

	/** The next requests fail: with a status, or as if the network were down. */
	failNext(...failures: (number | "network")[]): void {
		this.failures.push(...failures);
	}

	push(mutations: Mutation[]): PushResponse {
		this.pushes.push(mutations);
		const before = this.version;
		const results = mutations.map((m) => {
			const seen = this.results.get(m.mid);
			if (seen) return seen;
			const result: MutationResult = { mid: m.mid, ...this.apply(m) };
			this.results.set(m.mid, result);
			return result;
		});
		if (this.version > before) this.poke();
		return { results, version: this.version };
	}

	pull(cursor: number, limit = MAX_PULL): PullResponse {
		const after = [...this.versions.values()]
			.filter((row) => row.version > cursor)
			.sort((a, b) => a.version - b.version);
		const page = after.slice(0, limit);
		const changes = page.map((row): Change => {
			if (row.deleted) {
				return {
					entity: row.entity,
					key: row.key,
					version: row.version,
					deleted: true,
				};
			}
			const data = this.tables[row.entity].get(row.key);
			return {
				entity: row.entity,
				key: row.key,
				version: row.version,
				deleted: false,
				data,
			} as Change;
		});
		return {
			changes,
			cursor: page.at(-1)?.version ?? Math.max(cursor, this.version),
			more: after.length > page.length,
		};
	}

	me(): Account {
		const bookmarks = [...this.tables.bookmark.values()].length;
		const pro = this.plan === "pro";
		return {
			user_id: "user_fake",
			email: "someone@example.com",
			plan: this.plan,
			organizer: pro ? "cloud" : "device",
			limits: {
				bookmarks: this.bookmarkLimit,
				organized_per_month: pro ? 1000 : 0,
			},
			usage: { bookmarks, organized_this_month: 0 },
			billing: {
				status: pro ? "active" : "none",
				period_end: null,
				cancel_at_period_end: false,
				trial_available: !pro,
			},
		};
	}

	/** Tells connected clients the account changed, as the server does after a plan change. */
	accountChanged(): void {
		for (const socket of this.sockets)
			socket.deliver(JSON.stringify({ type: "account" }));
	}

	/** A `fetch` that answers the API's routes. */
	fetch = async (
		input: RequestInfo | URL,
		init?: RequestInit,
	): Promise<Response> => {
		const failure = this.failures.shift();
		if (failure === "network") throw new TypeError("Failed to fetch");
		if (failure)
			return json({ error: "fake", message: "Injected failure" }, failure);
		const url = new URL(String(input));
		const method = init?.method ?? "GET";
		if (method === "POST" && url.pathname === "/v1/sync/push") {
			const body = JSON.parse(String(init?.body)) as { mutations: Mutation[] };
			return json(this.push(body.mutations));
		}
		if (method === "GET" && url.pathname === "/v1/sync/pull") {
			const cursor = Number(url.searchParams.get("cursor") ?? 0);
			const limit = Number(url.searchParams.get("limit") ?? MAX_PULL);
			return json(this.pull(cursor, limit));
		}
		if (method === "GET" && url.pathname === "/v1/me") return json(this.me());
		if (method === "POST" && url.pathname === "/v1/organize") {
			if (this.plan !== "pro") {
				return json({ error: "forbidden", message: "Pro only" }, 403);
			}
			return json({ queued: 0, pending: 0 });
		}
		return json({ error: "not_found", message: url.pathname }, 404);
	};

	/** A socket factory that connects to this server, for the client's `socket` option. */
	socket: SocketFactory = () => {
		const socket = new FakeSocket(this.sockets);
		return socket;
	};

	save(): FakeServerState {
		const rows: FakeServerState["rows"] = [];
		const tombstones: Versioned[] = [];
		for (const row of this.versions.values()) {
			const data = this.tables[row.entity].get(row.key);
			if (row.deleted || !data) tombstones.push(row);
			else
				rows.push({
					entity: row.entity,
					key: row.key,
					version: row.version,
					data,
				});
		}
		return {
			version: this.version,
			rows,
			tombstones,
			results: [...this.results],
		};
	}

	private restore(state: FakeServerState): void {
		this.version = state.version;
		for (const row of state.rows) {
			(this.tables[row.entity] as Map<Key, unknown>).set(row.key, row.data);
			this.versions.set(id(row), { ...row, deleted: false });
		}
		for (const row of state.tombstones) this.versions.set(id(row), row);
		this.results = new Map(state.results);
	}

	private apply(m: Mutation): Omit<MutationResult, "mid"> {
		if (m.actor === "ai" && this.userOwns(m)) return { status: "ignored" };
		if (m.op === "delete") {
			const removed = deleteRow(this.tables, m.entity, m.key);
			// Children go without tombstones of their own: clients cascade.
			for (const row of removed) this.versions.delete(id(row));
			this.bump(m.entity, m.key, true);
			return { status: "applied" };
		}
		const creating = !this.tables[m.entity].has(m.key);
		if (
			creating &&
			m.entity === "bookmark" &&
			this.liveBookmarks() >= this.bookmarkLimit
		) {
			return { status: "rejected", reason: "quota" };
		}
		if (!putRow(this.tables, m.entity, m.key, m.data)) {
			return { status: "rejected", reason: "invalid" };
		}
		this.bump(m.entity, m.key, false);
		return { status: "applied" };
	}

	private userOwns(m: Mutation): boolean {
		if (m.entity !== "bookmark_tag" && m.entity !== "bookmark_folder")
			return false;
		const current = this.tables[m.entity].get(m.key);
		return (
			current !== undefined && (current.origin === "user" || current.excluded)
		);
	}

	private liveBookmarks(): number {
		return this.tables.bookmark.size;
	}

	private bump(entity: Entity, key: Key, deleted: boolean): void {
		this.version += 1;
		this.versions.set(`${entity}\u0000${key}`, {
			entity,
			key,
			version: this.version,
			deleted,
		});
	}

	/** Closes the oldest socket as the API does to make room for a newer one. */
	displaceOldestSocket(): void {
		const [oldest] = this.sockets;
		oldest?.close(1008);
	}

	private poke(): void {
		const message = JSON.stringify({ type: "poke", version: this.version });
		for (const socket of this.sockets) socket.deliver(message);
	}
}

const id = (row: { entity: Entity; key: Key }) =>
	`${row.entity}\u0000${row.key}`;

function json(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { "Content-Type": "application/json" },
	});
}

class FakeSocket implements SocketLike {
	onopen: (() => void) | null = null;
	onmessage: ((event: { data: unknown }) => void) | null = null;
	onclose: ((event: { code?: number }) => void) | null = null;
	onerror: (() => void) | null = null;

	constructor(private readonly all: Set<FakeSocket>) {
		all.add(this);
		queueMicrotask(() => this.onopen?.());
	}

	send(data: string): void {
		if (data === "ping") queueMicrotask(() => this.deliver("pong"));
	}

	close(code = 1000): void {
		if (!this.all.delete(this)) return;
		queueMicrotask(() => this.onclose?.({ code }));
	}

	deliver(data: string): void {
		queueMicrotask(() => this.onmessage?.({ data }));
	}
}
