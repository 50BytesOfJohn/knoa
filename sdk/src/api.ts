//! HTTP calls to the API Worker, with the session token on each.

import type {
	Account,
	ErrorResponse,
	OrganizeRequest,
	OrganizeResponse,
	PullResponse,
	PushRequest,
	PushResponse,
} from "./protocol";
import { MAX_PULL } from "./protocol";

export type TokenProvider = () => Promise<string | null>;

/** The server answered, but not with success. */
export class HttpError extends Error {
	constructor(
		readonly status: number,
		readonly code: string,
		message: string,
	) {
		super(message);
		this.name = "HttpError";
	}
}

/** There is no session token: the user is signed out, or Clerk isn't ready. */
export class SignedOutError extends Error {
	constructor() {
		super("Not signed in");
		this.name = "SignedOutError";
	}
}

/**
 * Worth trying again later: the network, the server having a bad moment, an expired token
 * or a rate limit. Anything else the server said is its final answer.
 */
export function isRetryable(error: unknown): boolean {
	if (error instanceof SignedOutError) return true;
	if (!(error instanceof HttpError)) return true;
	return (
		error.status >= 500 ||
		error.status === 401 ||
		error.status === 408 ||
		error.status === 429
	);
}

export class Api {
	constructor(
		private readonly baseUrl: string,
		private readonly getToken: TokenProvider,
		private readonly fetchFn: typeof fetch = (...args) => fetch(...args),
	) {}

	push(body: PushRequest): Promise<PushResponse> {
		return this.call("POST", "/v1/sync/push", body);
	}

	pull(cursor: number): Promise<PullResponse> {
		return this.call("GET", `/v1/sync/pull?cursor=${cursor}&limit=${MAX_PULL}`);
	}

	me(): Promise<Account> {
		return this.call("GET", "/v1/me");
	}

	organize(body: OrganizeRequest): Promise<OrganizeResponse> {
		return this.call("POST", "/v1/organize", body);
	}

	imageUrl(imageKey: string): string {
		return `${this.baseUrl}/img/${imageKey}`;
	}

	private async call<T>(method: string, path: string, body?: unknown) {
		const token = await this.getToken();
		if (!token) throw new SignedOutError();
		const headers: Record<string, string> = {
			Authorization: `Bearer ${token}`,
		};
		if (body !== undefined) headers["Content-Type"] = "application/json";
		const response = await this.fetchFn(`${this.baseUrl}${path}`, {
			method,
			headers,
			body: body === undefined ? undefined : JSON.stringify(body),
		});
		if (!response.ok) throw await httpError(response);
		return (await response.json()) as T;
	}
}

async function httpError(response: Response): Promise<HttpError> {
	try {
		const body = (await response.json()) as ErrorResponse;
		return new HttpError(response.status, body.error, body.message);
	} catch {
		return new HttpError(
			response.status,
			"http",
			`The server answered ${response.status}`,
		);
	}
}
