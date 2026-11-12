//! What each kind of bookmark is and which typed fields it has.

// The test script checks this packaged copy against crates/knoa-core/data/kinds.json.
import data from "./kinds.json";

export type FieldType =
	| "text"
	| "integer"
	| "number"
	| "date"
	| "duration"
	| "list";

export interface FieldDef {
	name: string;
	type: FieldType;
	description: string;
}

export interface KindDef {
	label: string;
	plural: string;
	description: string;
	/** The fields worth showing on a card, in order. */
	key_facts: string[];
	fields: FieldDef[];
}

export const KINDS = data as Record<string, KindDef>;

export function kindLabel(kind: string | null, plural = false): string {
	const def = kind ? KINDS[kind] : undefined;
	if (!def) return "Other";
	return plural ? def.plural : def.label;
}

/** `reading_minutes` → `Reading minutes`, as the desktop labels fields. */
export function fieldLabel(field: FieldDef): string {
	const label = field.name.replaceAll("_", " ");
	return label.charAt(0).toUpperCase() + label.slice(1);
}

/** The value as a short display string, or `""` when there is nothing to show. */
export function displayField(field: FieldDef, value: unknown): string {
	if (value == null) return "";
	if (field.type === "duration" && typeof value === "number") {
		return formatDuration(Math.round(value));
	}
	if (Array.isArray(value)) {
		return value.filter((item) => typeof item === "string").join(", ");
	}
	return String(value);
}

/** The bookmark's fields that have a value, labelled, in the kind's order. */
export function fieldsOf(
	kind: string | null,
	fields: Record<string, unknown>,
): { label: string; value: string }[] {
	const def = kind ? KINDS[kind] : undefined;
	if (!def) return [];
	return def.fields
		.map((field) => ({
			label: fieldLabel(field),
			value: displayField(field, fields[field.name]),
		}))
		.filter((entry) => entry.value !== "");
}

/** The kind's key facts that have a value, for a card's one line of detail. */
export function keyFacts(
	kind: string | null,
	fields: Record<string, unknown>,
): string[] {
	const def = kind ? KINDS[kind] : undefined;
	if (!def) return [];
	return def.key_facts.flatMap((name) => {
		const field = def.fields.find((f) => f.name === name);
		const text = field ? displayField(field, fields[name]) : "";
		return text ? [text] : [];
	});
}

/** `148` → `2h 28m`, `45` → `45 min`. */
export function formatDuration(minutes: number): string {
	const h = Math.trunc(minutes / 60);
	const m = minutes % 60;
	if (h === 0) return `${m} min`;
	if (m === 0) return `${h}h`;
	return `${h}h ${m}m`;
}
