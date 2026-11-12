//! The poke socket: the server says when there is something to pull.

import {
	PING,
	PONG,
	type ServerMessage,
	SOCKET_PROTOCOL,
	SOCKET_TOKEN_PREFIX,
} from "./protocol";
import type { TokenProvider } from "./api";
import { Backoff } from "./backoff";

/** The parts of the browser's `WebSocket` this uses, so tests and React Native can pass theirs. */
export interface SocketLike {
	onopen: (() => void) | null;
	onmessage: ((event: { data: unknown }) => void) | null;
	onclose: (() => void) | null;
	onerror: (() => void) | null;
	send(data: string): void;
	close(): void;
}

export type SocketFactory = (url: string, protocols: string[]) => SocketLike;

export interface PokeHandlers {
	/** Connected again: anything may have changed while it was down. */
	onOpen(): void;
	onPoke(version: number): void;
	onAccount(): void;
}

/** Proxies and load balancers drop quiet connections after about a minute. */
const PING_EVERY_MS = 25_000;
const PONG_WITHIN_MS = 10_000;

export class PokeSocket {
	private socket: SocketLike | null = null;
	private stopped = true;
	private retry: ReturnType<typeof setTimeout> | null = null;
	private ping: ReturnType<typeof setInterval> | null = null;
	private pongDue: ReturnType<typeof setTimeout> | null = null;
	private readonly backoff = new Backoff(1_000, 30_000);

	constructor(
		private readonly url: string,
		private readonly getToken: TokenProvider,
		private readonly create: SocketFactory,
		private readonly handlers: PokeHandlers,
	) {}

	start(): void {
		if (!this.stopped) return;
		this.stopped = false;
		void this.connect();
	}

	stop(): void {
		this.stopped = true;
		this.clearTimers();
		this.socket?.close();
		this.socket = null;
	}

	/** Skips the wait, for when the browser says it is back online. */
	reconnectNow(): void {
		if (this.stopped || this.socket) return;
		this.backoff.reset();
		if (this.retry) clearTimeout(this.retry);
		this.retry = null;
		void this.connect();
	}

	private async connect(): Promise<void> {
		let token: string | null = null;
		try {
			token = await this.getToken();
		} catch {
			token = null;
		}
		if (this.stopped) return;
		if (!token) return this.scheduleReconnect();

		const socket = this.create(this.url, [
			SOCKET_PROTOCOL,
			`${SOCKET_TOKEN_PREFIX}${token}`,
		]);
		this.socket = socket;
		socket.onopen = () => {
			this.backoff.reset();
			this.startPinging(socket);
			this.handlers.onOpen();
		};
		socket.onmessage = (event) => this.receive(event.data);
		socket.onerror = () => socket.close();
		socket.onclose = () => {
			if (this.socket !== socket) return;
			this.socket = null;
			this.clearTimers();
			this.scheduleReconnect();
		};
	}

	private receive(data: unknown): void {
		if (typeof data !== "string") return;
		if (data === PONG) {
			if (this.pongDue) clearTimeout(this.pongDue);
			this.pongDue = null;
			return;
		}
		let message: ServerMessage;
		try {
			message = JSON.parse(data) as ServerMessage;
		} catch {
			return;
		}
		if (message.type === "poke") this.handlers.onPoke(message.version);
		else if (message.type === "account") this.handlers.onAccount();
	}

	private startPinging(socket: SocketLike): void {
		this.ping = setInterval(() => {
			socket.send(PING);
			this.pongDue ??= setTimeout(() => socket.close(), PONG_WITHIN_MS);
		}, PING_EVERY_MS);
	}

	private scheduleReconnect(): void {
		if (this.stopped || this.retry) return;
		this.retry = setTimeout(() => {
			this.retry = null;
			void this.connect();
		}, this.backoff.next());
	}

	private clearTimers(): void {
		if (this.ping) clearInterval(this.ping);
		if (this.pongDue) clearTimeout(this.pongDue);
		if (this.retry) clearTimeout(this.retry);
		this.ping = null;
		this.pongDue = null;
		this.retry = null;
	}
}
