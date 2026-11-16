//! A workspace's folders as the tree the user sees, as `FolderTree` in knoa-core builds it.

import type { Folder, FolderId } from "./protocol";

/** Levels of folders: a top-level folder, its children and theirs. As in knoa-core. */
export const MAX_FOLDER_DEPTH = 3;

const PATH_SEPARATOR = " › ";

export interface FolderNode {
	folder: Folder;
	/** 0 at the top level. */
	depth: number;
	/** The parent it shows under, which may differ from the stored `parent_id`. */
	parent: FolderId | null;
}

/** Why a folder can't go where it was asked to. */
export type MoveError = "unknown_parent" | "into_itself" | "too_deep";

/**
 * One workspace's folders in display order: each folder followed by its subfolders,
 * siblings by position. A folder whose parent is missing, in a cycle, or would sit deeper
 * than {@link MAX_FOLDER_DEPTH} shows at the top level, so any rows make a valid tree.
 */
export class FolderTree {
	/** Every folder, in display order. */
	readonly nodes: readonly FolderNode[];
	private readonly index = new Map<FolderId, number>();
	/** One past the last node of each subtree, which is contiguous in display order. */
	private readonly ends: number[];

	constructor(folders: Folder[]) {
		const sorted = [...folders].sort(byPosition);
		this.nodes = layOut(sorted);
		this.nodes.forEach((node, at) => this.index.set(node.folder.id, at));
		this.ends = subtreeEnds(this.nodes);
	}

	get folders(): Folder[] {
		return this.nodes.map((node) => node.folder);
	}

	/** Whether any folder has a parent. A flat tree shows no hierarchy chrome. */
	get isNested(): boolean {
		return this.nodes.some((node) => node.depth > 0);
	}

	get(id: FolderId): Folder | undefined {
		return this.node(id)?.folder;
	}

	depth(id: FolderId): number {
		return this.node(id)?.depth ?? 0;
	}

	parent(id: FolderId): Folder | undefined {
		const parent = this.node(id)?.parent;
		return parent ? this.get(parent) : undefined;
	}

	/** The folders directly under `parent`, or the top-level ones for `null`. */
	children(parent: FolderId | null): Folder[] {
		return this.nodes
			.filter((node) => node.parent === parent)
			.map((node) => node.folder);
	}

	/** The folder followed by everything under it. Empty when it isn't here. */
	subtree(id: FolderId): Folder[] {
		const at = this.index.get(id);
		if (at === undefined) return [];
		return this.nodes.slice(at, this.ends[at]).map((node) => node.folder);
	}

	descendants(id: FolderId): Folder[] {
		return this.subtree(id).slice(1);
	}

	/** Levels in the folder's subtree: 1 for a folder with no subfolders. */
	height(id: FolderId): number {
		const at = this.index.get(id);
		if (at === undefined) return 0;
		const base = this.nodes[at]?.depth ?? 0;
		let height = 1;
		for (const node of this.nodes.slice(at, this.ends[at])) {
			height = Math.max(height, node.depth - base + 1);
		}
		return height;
	}

	/** The top-level folder down to this one. Empty when it isn't here. */
	path(id: FolderId): Folder[] {
		const path: Folder[] = [];
		let node = this.node(id);
		while (node) {
			path.unshift(node.folder);
			node = node.parent ? this.node(node.parent) : undefined;
		}
		return path;
	}

	/** "Wallpapers › Anime", for places that list folders flat. */
	pathName(id: FolderId): string {
		return this.path(id)
			.map((folder) => folder.name)
			.join(PATH_SEPARATOR);
	}

	/** Why a new folder can't go under `parent`, or null when it can. */
	checkNewChild(parent: FolderId | null): MoveError | null {
		return this.checkRoom(parent, 1);
	}

	/** Why `id` and its subfolders can't move under `parent`, or null when they can. */
	checkMove(id: FolderId, parent: FolderId | null): MoveError | null {
		if (parent && this.get(parent)?.workspace_id !== this.get(id)?.workspace_id) {
			return "unknown_parent";
		}
		if (parent && this.subtree(id).some((folder) => folder.id === parent)) {
			return "into_itself";
		}
		return this.checkRoom(parent, Math.max(this.height(id), 1));
	}

	private checkRoom(parent: FolderId | null, height: number): MoveError | null {
		let depth = 0;
		if (parent) {
			const node = this.node(parent);
			if (!node) return "unknown_parent";
			depth = node.depth + 1;
		}
		return depth + height > MAX_FOLDER_DEPTH ? "too_deep" : null;
	}

	private node(id: FolderId): FolderNode | undefined {
		const at = this.index.get(id);
		return at === undefined ? undefined : this.nodes[at];
	}
}

function byPosition(a: Folder, b: Folder): number {
	return a.position - b.position || a.name.localeCompare(b.name);
}

/**
 * Display order from stored parents. Folders too deep for their parent, then folders only
 * reachable through a cycle, start subtrees of their own at the top level.
 */
function layOut(folders: Folder[]): FolderNode[] {
	const byId = new Map(folders.map((folder) => [folder.id, folder]));
	const parents = new Map<FolderId, FolderId | null>();
	for (const folder of folders) {
		const parent = byId.get(folder.parent_id ?? "");
		parents.set(folder.id, parent?.workspace_id === folder.workspace_id ? parent.id : null);
	}
	detachCycles(parents);
	const children = new Map<FolderId, Folder[]>();
	const roots: Folder[] = [];
	for (const folder of folders) {
		const parent = parents.get(folder.id) ?? null;
		if (parent) {
			const siblings = children.get(parent) ?? [];
			siblings.push(folder);
			children.set(parent, siblings);
		} else {
			roots.push(folder);
		}
	}
	const nodes: FolderNode[] = [];
	const placed = new Set<FolderId>();
	const tooDeep: Folder[] = [];
	const visit = (folder: Folder, parent: FolderId | null, depth: number) => {
		placed.add(folder.id);
		nodes.push({ folder, depth, parent });
		for (const child of children.get(folder.id) ?? []) {
			if (placed.has(child.id)) continue;
			if (depth + 1 >= MAX_FOLDER_DEPTH) tooDeep.push(child);
			else visit(child, folder.id, depth + 1);
		}
	};
	for (const root of roots) visit(root, null, 0);
	for (;;) {
		const next =
			tooDeep.find((folder) => !placed.has(folder.id)) ??
			folders.find((folder) => !placed.has(folder.id));
		if (!next) return nodes;
		visit(next, null, 0);
	}
}

/** Each subtree runs until the next node at its depth or above. */
function subtreeEnds(nodes: readonly FolderNode[]): number[] {
	const ends = nodes.map(() => nodes.length);
	const open: number[] = [];
	nodes.forEach((node, at) => {
		while (open.length > 0) {
			const last = open[open.length - 1] as number;
			if ((nodes[last]?.depth ?? 0) < node.depth) break;
			ends[last] = at;
			open.pop();
		}
		open.push(at);
	});
	return ends;
}

/** Every member of a cycle is a root, as in knoa-core. */
function detachCycles(parents: Map<FolderId, FolderId | null>): void {
	const visited = new Set<FolderId>();
	for (const start of parents.keys()) {
		const path: FolderId[] = [];
		let at: FolderId | null = start;
		while (at !== null) {
			if (visited.has(at)) {
				const cycle = path.indexOf(at);
				if (cycle >= 0) for (const member of path.slice(cycle)) parents.set(member, null);
				break;
			}
			visited.add(at);
			path.push(at);
			at = parents.get(at) ?? null;
		}
	}
}
