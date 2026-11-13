//! The fake server's end of a client's WebSocket, delivering on microtasks like a real one.

import type { SocketLike } from "./socket";

export class FakeSocket implements SocketLike {
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
