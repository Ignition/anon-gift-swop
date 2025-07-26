import {
	encode_message_v2,
	decode_message_v2,
	encode_state_v2,
	decode_state_v2
} from '$lib/wasm/rust_wasm.js';

export interface CompactMessage {
	giver: string;
	receiver: string;
}

export interface CompactState {
	names: string[];
	forbidden_pairs: number[]; // Flattened array [a1,b1,a2,b2,...]
	history: number[]; // Flattened array of assignments
}

export interface EncodingComparison {
	compact: string;
	json: string;
	compactSize: number;
	jsonSize: number;
	reduction: number;
}

/**
 * Compact URL encoding utilities using base32 encoding
 * Base32 is human-friendly: case-insensitive, no confusing characters
 */
export const compactEncoding = {
	/**
	 * Encode a gift assignment message into a compact base32 string (versioned)
	 * V2 format includes version byte for future compatibility
	 * Example: "Alice" + "Bob" → "cnqglaaay6omzzcumyumokj7beaa" (28 chars)
	 * vs JSON: {"version":"1.0","data":{"giver":"Alice","receiver":"Bob"}} → ~80 chars
	 */
	encodeMessage(giver: string, receiver: string): string {
		const result = encode_message_v2(giver, receiver);
		if (typeof result === 'string') {
			return result;
		}
		throw new Error(result as string);
	},

	/**
	 * Decode a compact base32 string back to giver/receiver (auto-detects version)
	 */
	decodeMessage(encoded: string): CompactMessage {
		const result = decode_message_v2(encoded);
		if (result && typeof result === 'object' && 'giver' in result && 'receiver' in result) {
			return result as CompactMessage;
		}
		throw new Error('Failed to decode message');
	},

	/**
	 * Encode application state into a compact base32 string (versioned)
	 */
	encodeState(
		names: string[],
		forbiddenPairs: Array<[number, number]>,
		history: number[][]
	): string {
		// Flatten forbidden pairs
		const flattenedPairs = new Uint32Array(forbiddenPairs.flatMap(([a, b]) => [a, b]));

		// Flatten history
		const flattenedHistory = new Uint32Array(history.flat());

		const result = encode_state_v2(names, flattenedPairs, flattenedHistory);
		if (typeof result === 'string') {
			return result;
		}
		throw new Error(result as string);
	},

	/**
	 * Decode a compact base32 string back to application state (auto-detects version)
	 */
	decodeState(encoded: string): CompactState {
		const result = decode_state_v2(encoded);
		if (result && typeof result === 'object') {
			return result as CompactState;
		}
		throw new Error('Failed to decode state');
	},

	/**
	 * Create a shareable URL with compact encoding
	 */
	createShareableUrl(baseUrl: string, giver: string, receiver: string): string {
		const encoded = this.encodeMessage(giver, receiver);
		const url = new URL(baseUrl);
		url.searchParams.set('m', encoded); // 'm' for message (shorter param name)
		return url.toString();
	},

	/**
	 * Create a state URL with compact encoding
	 */
	createStateUrl(
		baseUrl: string,
		names: string[],
		forbiddenPairs: Array<[number, number]>,
		history: number[][]
	): string {
		const encoded = this.encodeState(names, forbiddenPairs, history);
		const url = new URL(baseUrl);
		url.searchParams.set('s', encoded); // 's' for state (shorter param name)
		return url.toString();
	},

	/**
	 * Parse message from URL
	 */
	parseMessageFromUrl(url: string | URL): CompactMessage | null {
		const urlObj = typeof url === 'string' ? new URL(url) : url;
		const encoded = urlObj.searchParams.get('m');
		if (!encoded) {
			return null;
		}
		try {
			return this.decodeMessage(encoded);
		} catch {
			return null;
		}
	},

	/**
	 * Parse state from URL
	 */
	parseStateFromUrl(url: string | URL): CompactState | null {
		const urlObj = typeof url === 'string' ? new URL(url) : url;
		const encoded = urlObj.searchParams.get('s');
		if (!encoded) {
			return null;
		}
		try {
			return this.decodeState(encoded);
		} catch {
			return null;
		}
	}
};
