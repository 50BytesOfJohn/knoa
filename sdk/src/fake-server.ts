//! An in-memory stand-in for the API Worker, with just enough of the protocol for the
//! client's tests and for working on the web app without the real API. Not for production.

import {
	type Account,
	BILLING_ERROR,
	type BillingCheckoutRequest,
	type Change,
	type Entity,
	type Key,
	LANGUAGES,
	type Language,
	type LanguageRequest,
	MAX_PULL,
	type Mutation,
	type MutationResult,
	type OrganizeRequest,
	type OrganizeResponse,
	PLAN_LIMITS,
	type PullResponse,
	type PushResponse,
	type Rows,
} from "./protocol";
import { FakeSocket } from "./fake-socket";
import type { SocketFactory } from "./socket";
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
	language: Language | null = null;
	/** Full runs and folders-only runs queued this month, as the server counts them. */
	organized = 0;
	foldersOnly = 0;
	/** Bookmarks waiting for the organizer, which never runs here: each is charged once. */
	private queued = new Set<Key>();
	/** What a subscriber asked for beyond the month's room, kept to queue later. */
	private deferred = new Set<Key>();

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
		const created: Key[] = [];
		const results = mutations.map((m) => {
			const seen = this.results.get(m.mid);
			if (seen) return seen;
			const creating = isNewBookmark(m) && !this.tables.bookmark.has(m.key);
			const result: MutationResult = { mid: m.mid, ...this.apply(m) };
			this.results.set(m.mid, result);
			if (creating && result.status === "applied") created.push(m.key);
			return result;
		});
		// As the API does, new bookmarks of a cloud account are queued without asking.
		if (this.me().organizer === "cloud") this.queue(created, false);
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
			organizer: pro || this.room(false) > 0 ? "cloud" : "device",
			limits: {
				bookmarks: this.bookmarkLimit,
				organized_per_month: PLAN_LIMITS[this.plan].organized_per_month,
			},
			usage: { bookmarks, organized_this_month: this.organized },
			billing: {
				status: pro ? "active" : "none",
				period_end: null,
				cancel_at_period_end: false,
				trial_available: !pro,
			},
			language: this.language,
		};
	}

	/**
	 * Queues what fits this month's room. A subscriber's server keeps the rest as pending;
	 * a free account's refuses it, and organizing goes back to the user's own devices once
	 * the full-run room is used up. For a `bookmarks` scope it says which ids it took on.
	 */
	organize({ scope, folders_only }: OrganizeRequest): OrganizeResponse {
		const before = this.version;
		const asked = "bookmarks" in scope ? scope.bookmarks : this.inScope(scope);
		const { queued, accepted } = this.queue(asked, folders_only);
		if (this.version > before) this.poke();
		const pending = this.plan === "pro" ? this.deferred.size : 0;
		return "bookmarks" in scope
			? { queued, pending, accepted }
			: { queued, pending };
	}

	/** Queues what fits the room, charging a bookmark only while it isn't queued yet. */
	private queue(
		keys: Key[],
		foldersOnly: boolean,
	): { queued: number; accepted: Key[] } {
		const organizer = this.me().organizer;
		let queued = 0;
		const accepted: Key[] = [];
		for (const key of keys) {
			const bookmark = this.tables.bookmark.get(key);
			if (!bookmark || bookmark.deleted_at !== null || this.queued.has(key)) {
				accepted.push(key);
			} else if (this.room(foldersOnly) > 0) {
				if (foldersOnly) this.foldersOnly += 1;
				else this.organized += 1;
				this.queued.add(key);
				this.deferred.delete(key);
				this.tables.bookmark.set(key, { ...bookmark, status: "queued" });
				this.bump("bookmark", key, false);
				queued += 1;
				accepted.push(key);
			} else if (this.plan === "pro") {
				this.deferred.add(key);
				accepted.push(key);
			}
		}
		if (this.me().organizer !== organizer) this.accountChanged();
		return { queued, accepted };
	}

	/** A free account's folders-only runs need room for full runs too. */
	private room(foldersOnly: boolean): number {
		const limits = PLAN_LIMITS[this.plan];
		const full = Math.max(0, limits.organized_per_month - this.organized);
		if (!foldersOnly) return full;
		if (this.plan === "free" && full === 0) return 0;
		return Math.max(0, limits.folders_only_per_month - this.foldersOnly);
	}

	/** The live bookmarks a workspace or `all` scope names. */
	private inScope(scope: { workspace: string } | { all: true }): Key[] {
		return [...this.tables.bookmark.entries()]
			.filter(
				([, b]) =>
					b.deleted_at === null &&
					(!("workspace" in scope) || b.workspace_id === scope.workspace),
			)
			.map(([key]) => key);
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
		if (method === "PUT" && url.pathname === "/v1/me/language") {
			return this.setLanguage(init);
		}
		if (method === "POST" && url.pathname === "/v1/organize") {
			return json(this.organize(JSON.parse(String(init?.body))));
		}
		if (method === "POST" && url.pathname.startsWith("/v1/billing/")) {
			return this.billing(url.pathname.slice("/v1/billing/".length), init);
		}
		return json({ error: "not_found", message: url.pathname }, 404);
	};

	private setLanguage(init?: RequestInit): Response {
		const { language } = JSON.parse(String(init?.body)) as LanguageRequest;
		if (!LANGUAGES.includes(language)) {
			return json({ error: "bad_request", message: "Unknown language" }, 400);
		}
		this.language = language;
		this.accountChanged();
		return json(this.me());
	}

	/** Checkout succeeds at once, with no payment page in between. */
	private billing(route: string, init?: RequestInit): Response {
		if (route === "refresh") return json({ plan: this.plan });
		if (route === "portal") {
			if (this.plan !== "pro") {
				return json(
					{ error: BILLING_ERROR.noSubscription, message: "No subscription" },
					404,
				);
			}
			return json({ url: "/app" });
		}
		if (route !== "checkout")
			return json({ error: "not_found", message: route }, 404);
		const { period } = JSON.parse(String(init?.body)) as BillingCheckoutRequest;
		if (period !== "month" && period !== "annual") {
			return json({ error: "bad_request", message: "Unknown period" }, 400);
		}
		if (this.plan === "pro") {
			return json(
				{ error: BILLING_ERROR.alreadySubscribed, message: "Already Pro" },
				409,
			);
		}
		this.plan = "pro";
		this.accountChanged();
		return json({ url: "/app?checkout=success" });
	}

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
			const parent =
				m.entity === "folder"
					? (this.tables.folder.get(m.key)?.parent_id ?? null)
					: null;
			const removed = deleteRow(this.tables, m.entity, m.key);
			for (const row of removed) {
				if (row.entity === m.entity && row.key === m.key) continue;
				// As the protocol says: only a bookmark's children get tombstones.
				if (m.entity === "bookmark") this.bump(row.entity, row.key, true);
				else this.versions.delete(id(row));
			}
			this.bump(m.entity, m.key, true);
			if (m.entity === "folder") this.moveUp(m.key, parent);
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
		// As the API does, a row's id is its key.
		const data = this.withoutOverruledSummary(m);
		const keyed = ID_KEYED.has(m.entity) ? { ...data, id: m.key } : data;
		if (!putRow(this.tables, m.entity, m.key, keyed)) {
			return { status: "rejected", reason: "invalid" };
		}
		this.bump(m.entity, m.key, false);
		return { status: "applied" };
	}

	/** A deleted folder's subfolders move to its parent, as the protocol says. */
	private moveUp(deleted: string, parent: string | null): void {
		for (const [key, folder] of this.tables.folder) {
			if (folder.parent_id !== deleted) continue;
			this.tables.folder.set(key, { ...folder, parent_id: parent });
			this.bump("folder", key, false);
		}
	}

	/** The AI's summary gives way to the user's; the rest of its put still applies. */
	private withoutOverruledSummary(m: Mutation & { op: "put" }): typeof m.data {
		if (m.actor !== "ai" || m.entity !== "bookmark") return m.data;
		if (this.tables.bookmark.get(m.key)?.summary_origin !== "user") return m.data;
		const { summary: _, summary_origin: __, ...rest } = m.data;
		return rest;
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

/** A user's put of a bookmark not organized yet, which the API queues on arrival. */
function isNewBookmark(m: Mutation): m is Mutation & { op: "put" } {
	return (
		m.op === "put" &&
		m.entity === "bookmark" &&
		m.actor !== "ai" &&
		(m.data as Rows["bookmark"]).status !== "organized"
	);
}

const ID_KEYED: ReadonlySet<Entity> = new Set(["workspace", "folder", "bookmark"]);

const id = (row: { entity: Entity; key: Key }) =>
	`${row.entity}\u0000${row.key}`;

function json(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { "Content-Type": "application/json" },
	});
}
