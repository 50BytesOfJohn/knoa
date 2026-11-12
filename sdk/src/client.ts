//! The sync client: keeps the replica in step with the server and tells the UI when it changed.

import type {
	Account,
	Mutation,
	OrganizeRequest,
	OrganizeResponse,
} from "./protocol";
import {
	Api,
	HttpError,
	isRetryable,
	SignedOutError,
	type TokenProvider,
} from "./api";
import { Backoff } from "./backoff";
import { nextBatch } from "./batch";
import type { Library } from "./library";
import { type Rejection, Replica } from "./replica";
import { PokeSocket, type SocketFactory, type SocketLike } from "./socket";
import type { Storage } from "./storage";

export type SyncStatus = "loading" | "offline" | "syncing" | "synced" | "error";

export interface SyncError {
	kind: "quota" | "rejected" | "signed-out" | "server";
	message: string;
}

export interface SyncSnapshot {
	status: SyncStatus;
	/** The last thing that went wrong, until it's dismissed or sync succeeds again. */
	error: SyncError | null;
	/** The library has been loaded from storage or pulled at least once. */
	ready: boolean;
	/** Local changes the server hasn't answered yet. */
	pending: number;
	account: Account | null;
	library: Library;
}

export interface SyncOptions {
	apiUrl: string;
	getToken: TokenProvider;
	storage: Storage;
	fetch?: typeof fetch;
	/** How to open the poke socket; `null` to go without one. Defaults to `WebSocket`. */
	socket?: SocketFactory | null;
	/** Pull when the tab regains focus or the browser comes back online. Defaults to on. */
	watchBrowser?: boolean;
}

export class SyncClient {
	readonly api: Api;
	private readonly replica = new Replica();
	private readonly storage: Storage;
	private readonly socket: PokeSocket | null;
	private readonly listeners = new Set<() => void>();
	private readonly backoff = new Backoff(1_000, 60_000);
	private snapshot: SyncSnapshot;
	private running: Promise<void> | null = null;
	private again = false;
	private retry: ReturnType<typeof setTimeout> | null = null;
	private saving: Promise<void> = Promise.resolve();
	private unwatch: (() => void) | null = null;
	private stopped = false;

	constructor(private readonly options: SyncOptions) {
		// Checked before every request: a stopped client never sends again.
		const getToken: TokenProvider = async () =>
			this.stopped ? null : options.getToken();
		this.api = new Api(options.apiUrl, getToken, options.fetch);
		this.storage = options.storage;
		const factory =
			options.socket === undefined ? browserSocket() : options.socket;
		this.socket = factory
			? new PokeSocket(socketUrl(options.apiUrl), getToken, factory, {
					onOpen: () => this.sync(),
					onPoke: (version) => {
						if (version > this.replica.cursor) this.sync();
					},
					onAccount: () => void this.refreshAccount(),
				})
			: null;
		this.snapshot = {
			status: "loading",
			error: null,
			ready: false,
			pending: 0,
			account: null,
			library: this.replica.library(),
		};
	}

	/** Renders from storage first, then syncs. */
	async start(): Promise<void> {
		const stored = await this.storage.load().catch(() => null);
		if (stored) this.replica.load(stored);
		this.update({ ready: stored !== null && stored.cursor > 0 });
		this.sync();
		void this.refreshAccount();
		this.socket?.start();
		if (this.options.watchBrowser !== false) this.unwatch = watchBrowser(this);
	}

	/** Stops for good: a round already running ends before its next request. */
	stop(): void {
		this.stopped = true;
		this.socket?.stop();
		this.unwatch?.();
		if (this.retry) clearTimeout(this.retry);
		this.retry = null;
	}

	subscribe = (listener: () => void): (() => void) => {
		this.listeners.add(listener);
		return () => this.listeners.delete(listener);
	};

	getSnapshot = (): SyncSnapshot => this.snapshot;

	/** Shows the change at once and queues it for the server. */
	mutate(mutations: Mutation[]): void {
		if (mutations.length === 0) return;
		this.replica.enqueue(mutations);
		this.persist();
		this.update({});
		this.sync();
	}

	/** Pushes what's pending, then pulls until up to date. Calls made meanwhile coalesce. */
	sync(): void {
		if (this.stopped) return;
		this.again = true;
		if (this.running) return;
		if (this.retry) clearTimeout(this.retry);
		this.retry = null;
		this.running = this.run().finally(() => {
			this.running = null;
		});
	}

	/** Resolves once the current round of syncing is over. For tests and for "Sync now". */
	async settled(): Promise<void> {
		while (this.running) await this.running;
	}

	dismissError(): void {
		this.update({ error: null });
	}

	async refreshAccount(): Promise<void> {
		try {
			this.update({ account: await this.api.me() });
		} catch {
			// The account card waits for the next try; sync reports connection trouble.
		}
	}

	organize(request: OrganizeRequest): Promise<OrganizeResponse> {
		return this.api.organize(request);
	}

	/** Called by {@link watchBrowser}. */
	wentOffline(): void {
		this.update({ status: "offline" });
	}

	/** Called by {@link watchBrowser}. */
	cameOnline(): void {
		this.socket?.reconnectNow();
		this.sync();
	}

	private async run(): Promise<void> {
		while (this.again && !this.stopped) {
			this.again = false;
			this.update({ status: "syncing" });
			try {
				const pushedBookmarks = await this.pushThenPull();
				this.backoff.reset();
				this.update({
					status: "synced",
					ready: true,
					error: untilDismissed(this.snapshot.error),
				});
				if (pushedBookmarks) void this.refreshAccount();
			} catch (error) {
				this.fail(error);
				return;
			}
		}
	}

	/**
	 * A push the server keeps failing still lets everyone else's changes in: the pull runs
	 * anyway, and the push's error is the one reported.
	 */
	private async pushThenPull(): Promise<boolean> {
		let pushed = false;
		let pushError: HttpError | null = null;
		try {
			pushed = await this.pushAll();
		} catch (error) {
			if (!(error instanceof HttpError) || error.status < 500) throw error;
			pushError = error;
		}
		await this.pullAll();
		if (pushError) throw pushError;
		return pushed;
	}

	/** Pushes in order, a batch at a time. Returns whether any bookmark was added or removed. */
	private async pushAll(): Promise<boolean> {
		let bookmarks = false;
		while (this.replica.pending.length > 0) {
			const batch = nextBatch(this.replica.pending);
			try {
				const response = await this.api.push({ mutations: batch });
				this.report(this.replica.settle(response.results));
				if (this.replica.pending[0] === batch[0]) {
					throw new HttpError(502, "protocol", "The push went unanswered");
				}
			} catch (error) {
				if (isRetryable(error)) throw error;
				// The server refused the request as a whole: final, like a rejection.
				const dropped = this.replica.discard(new Set(batch.map((m) => m.mid)));
				this.report(
					dropped.map((mutation) => ({ mutation, reason: "invalid" })),
				);
			}
			bookmarks ||= batch.some((m) => m.entity === "bookmark");
			this.persist();
			this.update({});
		}
		return bookmarks;
	}

	private async pullAll(): Promise<void> {
		let more = true;
		while (more) {
			const page = await this.api.pull(this.replica.cursor);
			this.replica.applyPull(page.changes, page.cursor);
			more = page.more;
			this.persist();
			this.update({ ready: true });
		}
	}

	private report(rejections: Rejection[]): void {
		const quota = rejections.some((r) => r.reason === "quota");
		const rejected = rejections.some((r) => r.reason === "invalid");
		if (quota) {
			this.update({
				error: {
					kind: "quota",
					message:
						"Your library is at your plan's bookmark limit, so the newest links weren't saved.",
				},
			});
			void this.refreshAccount();
		} else if (rejected) {
			this.update({
				error: {
					kind: "rejected",
					message: "The server turned down a change, so it was undone.",
				},
			});
		}
	}

	private fail(error: unknown): void {
		if (this.stopped) return;
		const offline =
			typeof navigator !== "undefined" && navigator.onLine === false;
		if (error instanceof SignedOutError) {
			this.update({
				status: "error",
				error: { kind: "signed-out", message: "You're signed out." },
			});
		} else if (offline || !(error instanceof HttpError)) {
			this.update({ status: "offline" });
		} else {
			this.update({
				status: "error",
				error: { kind: "server", message: error.message },
			});
		}
		if (isRetryable(error)) {
			this.retry = setTimeout(() => {
				this.retry = null;
				this.sync();
			}, this.backoff.next());
		}
	}

	private persist(): void {
		const change = this.replica.takeChanges();
		if (!change) return;
		this.saving = this.saving
			.then(() => this.storage.save(change))
			.catch(() => {
				// Storage full or blocked: the library still works, and the next save tries
				// these rows again.
				this.replica.unsaved(change);
			});
	}

	private update(patch: Partial<SyncSnapshot>): void {
		this.snapshot = {
			...this.snapshot,
			...patch,
			pending: this.replica.pending.length,
			library: this.replica.library(),
		};
		for (const listener of this.listeners) listener();
	}
}

/** Trouble connecting clears itself once sync works; a lost change stays until seen. */
function untilDismissed(error: SyncError | null): SyncError | null {
	return error?.kind === "quota" || error?.kind === "rejected" ? error : null;
}

function socketUrl(apiUrl: string): string {
	return `${apiUrl.replace(/^http/u, "ws")}/v1/sync/socket`;
}

function browserSocket(): SocketFactory | null {
	if (typeof WebSocket === "undefined") return null;
	return (url, protocols) => new WebSocket(url, protocols) as SocketLike;
}

function watchBrowser(client: SyncClient): (() => void) | null {
	if (typeof window === "undefined") return null;
	const pull = () => client.sync();
	const visible = () => {
		if (document.visibilityState === "visible") client.sync();
	};
	const online = () => client.cameOnline();
	const offline = () => client.wentOffline();
	window.addEventListener("focus", pull);
	window.addEventListener("online", online);
	window.addEventListener("offline", offline);
	document.addEventListener("visibilitychange", visible);
	return () => {
		window.removeEventListener("focus", pull);
		window.removeEventListener("online", online);
		window.removeEventListener("offline", offline);
		document.removeEventListener("visibilitychange", visible);
	};
}
