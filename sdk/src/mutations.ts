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
import { folderTree } from "./select";
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

/**
 * A folder last among `parent`'s children, or at the top level. Check the place first with
 * `folderTree(...).checkNewChild(parent)`: the server refuses a parent too deep.
 */
export function createFolder(
	workspace: WorkspaceId,
	name: string,
	position: number,
	color: number,
	parent: FolderId | null = null,
): { id: FolderId; mutation: Mutation } {
	const id = uuidv4();
	// Without `parent_id` at the top level: see `Folder.parent_id`.
	const row: Omit<Folder, "parent_id"> & Partial<Pick<Folder, "parent_id">> = {
		id,
		workspace_id: workspace,
		...(parent ? { parent_id: parent } : {}),
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

/**
 * Moves the folder and everything under it to `parent`, at `position` among its new
 * siblings. Check the move first with `folderTree(...).checkMove(id, parent)`.
 */
export function moveFolder(
	id: FolderId,
	parent: FolderId | null,
	position: number,
): Mutation {
	return put("folder", id, { parent_id: parent, position });
}

/** Deletes the folder and every folder under it, deepest first. Bookmarks stay. */
export function deleteFolder(library: Library, id: FolderId): Mutation[] {
	const folder = library.folder.get(id);
	if (!folder) return [remove("folder", id)];
	const subtree = folderTree(library, folder.workspace_id).subtree(id);
	return subtree.reverse().map((f) => remove("folder", f.id));
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
		summary_origin: "ai",
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

/**
 * The user's own title and description, sending only what changed, so fixing a title
 * leaves the AI's description to the AI. `null` when nothing changed or the title is
 * empty. An emptied description stays empty: the AI never writes over the user's.
 */
export function editBookmark(
	bookmark: Bookmark,
	edit: { title: string; summary: string },
): Mutation | null {
	const title = edit.title.trim();
	const summary = edit.summary.trim() || null;
	if (title === "") return null;
	const data: Partial<Bookmark> = {};
	if (title !== bookmark.title) data.title = title;
	if (summary !== bookmark.summary) {
		data.summary = summary;
		data.summary_origin = "user";
	}
	return Object.keys(data).length > 0 ? put("bookmark", bookmark.id, data) : null;
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
