/**
 * The Knoa sync protocol, version 1: the contract between the API Worker and every client
 * (the Rust desktop app and the TypeScript web app, extension and phone apps).
 *
 * Every client keeps a local replica and reads only from it. It pushes its own changes as
 * mutations, pulls everyone's changes by version, and is poked over a WebSocket when there
 * is something to pull.
 *
 * Wire format: JSON, snake_case. Times are Unix seconds, UTC. Ids are lowercase UUIDs.
 */

export const PROTOCOL_VERSION = 1;

// ── Ids ────────────────────────────────────────────────────────────────────────────────

/** Random (v4). Minted by the client that creates the workspace. */
export type WorkspaceId = string;
/** Random (v4). Minted by the client that creates the folder. */
export type FolderId = string;
/**
 * UUID v5 of the URL, with the workspace id as the namespace, so two devices that import
 * the same bookmark make the same row.
 */
export type BookmarkId = string;

// ── Rows ───────────────────────────────────────────────────────────────────────────────

export type Origin = "ai" | "user";

/** `queued`: waiting for the cloud organizer. Devices keep their own queue states local. */
export type BookmarkStatus = "new" | "queued" | "organized" | "failed";

export interface ListedTag {
	tag: string;
	description: string;
}

export interface Workspace {
	id: WorkspaceId;
	name: string;
	/** Index into the 12 inks. */
	color: number;
	position: number;
	instructions: string;
	/** Whether the guideline is on. */
	screening: boolean;
	guideline: string;
	only_listed_tags: boolean;
	/** At most 40 tags, each hint at most 60 characters. */
	tag_list: ListedTag[];
}

export interface Folder {
	id: FolderId;
	workspace_id: WorkspaceId;
	name: string;
	description: string;
	auto_assign: boolean;
	color: number;
	/** 45..=330 degrees; wins over `color`. */
	hue: number | null;
	icon: string | null;
	position: number;
}

export interface Bookmark {
	id: BookmarkId;
	workspace_id: WorkspaceId;
	url: string;
	title: string;
	/** Host without `www.`. */
	site: string;
	added_at: number;
	up_next: boolean;
	/** The user kept it or added it by hand: the guideline never sets it aside. */
	kept: boolean;
	/** When it went to Trash; null while in the library. */
	deleted_at: number | null;
	browser_folder: string | null;

	// Written by whoever organizes: the cloud organizer or, for free accounts, a device.
	status: BookmarkStatus;
	kind: string | null;
	summary: string | null;
	fields: Record<string, unknown>;
	extras: [name: string, value: string][];
	set_aside: string | null;
	model: string | null;
	organized_at: number | null;
	error: string | null;
	/** Path of the thumbnail under `GET /img/`; null without one. */
	image_key: string | null;
	image_width: number | null;
	image_height: number | null;
}

/** One row per (bookmark, tag): either the tag is on the bookmark, or the user removed it. */
export interface BookmarkTag {
	bookmark_id: BookmarkId;
	tag: string;
	origin: Origin;
	/** The user removed it; the AI must not add it back. */
	excluded: boolean;
}

/** One row per (bookmark, folder), like {@link BookmarkTag}. */
export interface BookmarkFolder {
	bookmark_id: BookmarkId;
	folder_id: FolderId;
	origin: Origin;
	probability: number | null;
	excluded: boolean;
}

/** The page's readable text, for full-text search. Only pulled by clients that ask. */
export interface BookmarkText {
	bookmark_id: BookmarkId;
	text: string;
}

export interface Rows {
	workspace: Workspace;
	folder: Folder;
	bookmark: Bookmark;
	bookmark_tag: BookmarkTag;
	bookmark_folder: BookmarkFolder;
	bookmark_text: BookmarkText;
}

export type Entity = keyof Rows;

export const ENTITIES = [
	"workspace",
	"folder",
	"bookmark",
	"bookmark_tag",
	"bookmark_folder",
	"bookmark_text",
] as const satisfies readonly Entity[];

/**
 * A row's key within its entity: the id, or for the link rows `<bookmark_id>/<tag>` and
 * `<bookmark_id>/<folder_id>`. For `bookmark_text` it is the bookmark id.
 */
export type Key = string;

export const tagKey = (bookmark: BookmarkId, tag: string): Key =>
	`${bookmark}/${tag}`;
export const folderLinkKey = (bookmark: BookmarkId, folder: FolderId): Key =>
	`${bookmark}/${folder}`;

// ── Limits ─────────────────────────────────────────────────────────────────────────────

/**
 * The longest value the server takes per text column, in characters (Unicode code points).
 * A longer one gets the mutation `rejected` as `invalid`. Generous: clients keep values
 * well inside these.
 */
export const MAX_CHARS = {
	workspace: { name: 200, instructions: 10_000, guideline: 10_000 },
	folder: { name: 200, description: 2_000, icon: 100 },
	bookmark: {
		url: 8_192,
		title: 2_000,
		site: 300,
		browser_folder: 2_000,
		kind: 100,
		summary: 4_000,
		set_aside: 2_000,
		model: 200,
		error: 4_000,
	},
	bookmark_text: { text: 100_000 },
} as const;

/** A tag, after normalizing. A longer one in a `bookmark_tag` key is `invalid`. */
export const MAX_TAG_CHARS = 40;
/** A workspace's `tag_list` keeps its first 40 tags and cuts each hint to 60 characters. */
export const MAX_LISTED_TAGS = 40;
export const MAX_TAG_DESCRIPTION_CHARS = 60;
/** `bookmark.fields` and `bookmark.extras` each, as UTF-8 JSON. */
export const MAX_JSON_BYTES = 32 * 1024;

/** Creating a row past these counts is `invalid`. Folders count per workspace. */
export const MAX_WORKSPACES = 100;
export const MAX_FOLDERS_PER_WORKSPACE = 500;
/** Tag rows per bookmark, removed (`excluded`) ones included. */
export const MAX_TAGS_PER_BOOKMARK = 100;

// ── Push ───────────────────────────────────────────────────────────────────────────────

/**
 * One change a client made. The server applies mutations in the order they arrive, and the
 * last one to arrive wins, column by column.
 *
 * - `put` creates the row or changes the columns in `data`. Columns left out keep their
 *   value. Creating a row needs every column that has no default (see the server).
 *   A `put` on a deleted row brings it back as a new row.
 * - `delete` leaves a tombstone. Deleting a bookmark also deletes its tags, folder links
 *   and text; a folder, its links; a workspace, its folders and bookmarks. Clients do the
 *   same when they pull the tombstone, so the children get no tombstones of their own.
 *
 * A user's value beats the AI's: an `ai` mutation never changes or deletes a tag or folder
 * link whose origin is `user` or that is excluded, and never adds a link to a folder that
 * is gone. Such a mutation is answered `ignored`.
 */
export type Mutation = {
	[E in Entity]: {
		/** Made by the client (UUID), so sending it twice is harmless. */
		mid: string;
		entity: E;
		key: Key;
		actor: Origin;
	} & ({ op: "put"; data: Partial<Rows[E]> } | { op: "delete" });
}[Entity];

export interface PushRequest {
	/** At most {@link MAX_PUSH} mutations. */
	mutations: Mutation[];
}

export type MutationStatus = "applied" | "ignored" | "rejected";

/** `quota`: the account is at its bookmark limit. `invalid`: the mutation makes no sense. */
export type RejectReason = "quota" | "invalid";

export interface MutationResult {
	mid: string;
	status: MutationStatus;
	reason?: RejectReason;
}

export interface PushResponse {
	/** One per mutation, in order. Anything but a network or 5xx error is final: drop it. */
	results: MutationResult[];
	/** The library's version after this push. */
	version: number;
}

export const MAX_PUSH = 500;
/** The largest push request body, as UTF-8 JSON; a larger one is answered 413. */
export const MAX_PUSH_BYTES = 4 * 1024 * 1024;

// ── Pull ───────────────────────────────────────────────────────────────────────────────

/**
 * A row as of `version`. Every write gives its row the library's next version, so a client
 * that has applied everything up to its cursor only needs what comes after.
 */
export type Change = {
	[E in Entity]: { entity: E; key: Key; version: number } & (
		| { deleted: false; data: Rows[E] }
		| { deleted: true }
	);
}[Entity];

/** `GET /v1/sync/pull?cursor=0&limit=1000&texts=1`. `texts=1` includes `bookmark_text`. */
export interface PullResponse {
	/** Ordered by version. Parents can come after their children: apply without assuming order. */
	changes: Change[];
	/** Pass as the next `cursor`. */
	cursor: number;
	/** There is more: pull again now. A page can hold fewer than `limit` changes when its rows are large. */
	more: boolean;
}

export const MAX_PULL = 1000;

// ── Poke ───────────────────────────────────────────────────────────────────────────────

/**
 * `GET /v1/sync/socket` upgrades to a WebSocket. Browsers can't set headers on it, so the
 * token travels as a subprotocol: `Sec-WebSocket-Protocol: knoa.v1, bearer.<token>`. The
 * server answers with `knoa.v1`.
 */
export const SOCKET_PROTOCOL = "knoa.v1";
export const SOCKET_TOKEN_PREFIX = "bearer.";

/** Server to client. `poke`: the library is at `version` now, pull if you are behind. */
export type ServerMessage =
	| { type: "poke"; version: number }
	| { type: "account" };

/** Client to server. The server answers the text `ping` with `pong`. */
export const PING = "ping";
export const PONG = "pong";

// ── Account ────────────────────────────────────────────────────────────────────────────

export type Plan = "free" | "pro";

/** `GET /v1/me`. An `account` message on the socket says this changed. */
export interface Account {
	user_id: string;
	email: string | null;
	plan: Plan;
	/**
	 * Who organizes this account's bookmarks. `cloud` for subscribers: devices leave their
	 * own organizer off. `device` for free accounts: a Mac with the user's own key does it
	 * and pushes the result.
	 */
	organizer: "cloud" | "device";
	limits: { bookmarks: number; organized_per_month: number };
	usage: { bookmarks: number; organized_this_month: number };
}

/**
 * `POST /v1/organize`, subscribers only (403 otherwise): run the cloud organizer again.
 * New bookmarks are queued without asking.
 */
export interface OrganizeRequest {
	scope:
		| { bookmarks: BookmarkId[] }
		| { workspace: WorkspaceId }
		| { all: true };
	/** Only ask the folder classifier again, over what the bookmark already has. */
	folders_only: boolean;
}

export interface OrganizeResponse {
	queued: number;
}

/**
 * `PUT /v1/images/<bookmark_id>` with the image as the body (png, jpeg, gif or webp, at
 * most 8 MiB) and `x-image-width` / `x-image-height` headers stores a thumbnail a device
 * fetched and sets the bookmark's image columns.
 */
export interface ImageResponse {
	image_key: string;
}

/**
 * `GET /v1/search?q=<text>&workspace=<id>&limit=50` (at most 200): full-text search on the
 * server, which also covers page text. Every word matches as a prefix; best match first.
 * Trashed and set-aside bookmarks are included: filter them on the client.
 */
export interface SearchResponse {
	bookmarks: BookmarkId[];
}

/** Every error response: `{ "error": "<code>", "message": "..." }`. */
export interface ErrorResponse {
	error: string;
	message: string;
}
