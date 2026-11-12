//! The changes a user can make, as protocol mutations.

import {
	type Bookmark,
	type BookmarkId,
	type Entity,
	type Folder,
	type FolderId,
	folderLinkKey,
	type Key,
	type Mutation,
	type Rows,
	tagKey,
	type Workspace,
	type WorkspaceId,
} from "./protocol";
import { v4 as uuidv4 } from "uuid";
import type { Library } from "./library";
import { bookmarkIdFor, parseLink, siteOf } from "./link";
import { MAX_TAG_CHARS, normalizeTag } from "./tag";

const now = () => Math.floor(Date.now() / 1000);

function put<E extends Entity>(
	entity: E,
	key: Key,
	data: Partial<Rows[E]>,
): Mutation {
	return {
		mid: uuidv4(),
		entity,
		key,
		actor: "user",
		op: "put",
		data,
	} as Mutation;
}

function remove(entity: Entity, key: Key): Mutation {
	return { mid: uuidv4(), entity, key, actor: "user", op: "delete" };
}

// ── Workspaces ─────────────────────────────────────────────────────────────────────────

export function createWorkspace(
	name: string,
	position: number,
	color = 0,
): { id: WorkspaceId; mutation: Mutation } {
	const id = uuidv4();
	const row: Workspace = {
		id,
		name: name.trim(),
		color,
		position,
		instructions: "",
		screening: false,
		guideline: "",
		only_listed_tags: false,
		tag_list: [],
	};
	return { id, mutation: put("workspace", id, row) };
}

export function updateWorkspace(
	id: WorkspaceId,
	changes: Partial<Omit<Workspace, "id">>,
): Mutation {
	return put("workspace", id, changes);
}

// ── Folders ────────────────────────────────────────────────────────────────────────────

export function createFolder(
	workspace: WorkspaceId,
	name: string,
	position: number,
	color: number,
): { id: FolderId; mutation: Mutation } {
	const id = uuidv4();
	const row: Folder = {
		id,
		workspace_id: workspace,
		name: name.trim(),
		description: "",
		auto_assign: true,
		color,
		hue: null,
		icon: null,
		position,
	};
	return { id, mutation: put("folder", id, row) };
}

export function renameFolder(id: FolderId, name: string): Mutation {
	return put("folder", id, { name: name.trim() });
}

export function deleteFolder(id: FolderId): Mutation {
	return remove("folder", id);
}

// ── Bookmarks ──────────────────────────────────────────────────────────────────────────

/**
 * A link the user added by hand. `null` when the text isn't a web address. Kept, so the
 * workspace's guideline never sets it aside. A link already in the library stays as it is,
 * or comes back from Trash.
 */
export function addLink(
	library: Library,
	workspace: WorkspaceId,
	input: string,
): { id: BookmarkId; url: string; mutations: Mutation[] } | null {
	const url = parseLink(input);
	if (!url) return null;
	const id = bookmarkIdFor(workspace, url);
	const existing = library.bookmark.get(id);
	if (existing) {
		return { id, url, mutations: existing.deleted_at ? [restore(id)] : [] };
	}
	const row: Bookmark = {
		id,
		workspace_id: workspace,
		url,
		title: url,
		site: siteOf(url),
		added_at: now(),
		up_next: false,
		kept: true,
		deleted_at: null,
		browser_folder: null,
		status: "new",
		kind: null,
		summary: null,
		fields: {},
		extras: [],
		set_aside: null,
		model: null,
		organized_at: null,
		error: null,
		image_key: null,
		image_width: null,
		image_height: null,
	};
	return { id, url, mutations: [put("bookmark", id, row)] };
}

export function setUpNext(id: BookmarkId, upNext: boolean): Mutation {
	return put("bookmark", id, { up_next: upNext });
}

/** Overrules the guideline: the bookmark stays in the library. */
export function keep(id: BookmarkId): Mutation {
	return put("bookmark", id, { kept: true, set_aside: null });
}

export function moveToTrash(id: BookmarkId): Mutation {
	return put("bookmark", id, { deleted_at: now() });
}

export function restore(id: BookmarkId): Mutation {
	return put("bookmark", id, { deleted_at: null });
}

export function deleteForGood(id: BookmarkId): Mutation {
	return remove("bookmark", id);
}

// ── Tags and folder links ──────────────────────────────────────────────────────────────

/** `null` when nothing is left of the tag once normalized, or it is too long. */
export function addTag(bookmark: BookmarkId, input: string): Mutation | null {
	const tag = normalizeTag(input);
	if (tag === "" || [...tag].length > MAX_TAG_CHARS) return null;
	return put("bookmark_tag", tagKey(bookmark, tag), {
		bookmark_id: bookmark,
		tag,
		origin: "user",
		excluded: false,
	});
}

/** Removing is remembered, so the AI doesn't add the tag back. */
export function removeTag(bookmark: BookmarkId, tag: string): Mutation {
	return put("bookmark_tag", tagKey(bookmark, tag), {
		bookmark_id: bookmark,
		tag,
		origin: "user",
		excluded: true,
	});
}

export function addToFolder(bookmark: BookmarkId, folder: FolderId): Mutation {
	return put("bookmark_folder", folderLinkKey(bookmark, folder), {
		bookmark_id: bookmark,
		folder_id: folder,
		origin: "user",
		probability: null,
		excluded: false,
	});
}

/** Removing is remembered, so the AI doesn't file it there again. */
export function removeFromFolder(
	bookmark: BookmarkId,
	folder: FolderId,
): Mutation {
	return put("bookmark_folder", folderLinkKey(bookmark, folder), {
		bookmark_id: bookmark,
		folder_id: folder,
		origin: "user",
		probability: null,
		excluded: true,
	});
}
