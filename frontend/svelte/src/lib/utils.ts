/**
 * Common utility functions for the Anonymous Gift Swap application
 */
import { devConsole } from './devConsole';

// Base64 encoding/decoding utilities
export const base64Utils = {
	/**
	 * Encodes an object to base64-encoded JSON string
	 */
	encodeJson(obj: unknown): string | null {
		try {
			const jsonString = JSON.stringify(obj);
			return btoa(jsonString);
		} catch (e) {
			devConsole.error('Failed to encode JSON to Base64:', e);
			return null;
		}
	},

	/**
	 * Decodes a base64-encoded JSON string back to an object
	 */
	decodeJson(base64: string): unknown | null {
		try {
			const jsonString = atob(base64);
			return JSON.parse(jsonString);
		} catch (e) {
			devConsole.error('Failed to decode or parse the base64 string:', e);
			return null;
		}
	},

	/**
	 * Encodes a plain text string to base64
	 */
	encodeString(text: string): string | null {
		try {
			return btoa(text);
		} catch (e) {
			devConsole.error('Failed to encode string to Base64:', e);
			return null;
		}
	},

	/**
	 * Decodes a base64 string to plain text
	 */
	decodeString(base64: string): string | null {
		try {
			return atob(base64);
		} catch (e) {
			devConsole.error('Failed to decode Base64 string:', e);
			return null;
		}
	}
};

// URL parameter utilities
export const urlUtils = {
	/**
	 * Gets a URL parameter value by name
	 */
	getParam(name: string): string | null {
		const urlParams = new URLSearchParams(window.location.search);
		return urlParams.get(name);
	},

	/**
	 * Sets a URL parameter and returns the updated URL
	 */
	setParam(baseUrl: string, name: string, value: string): string {
		const url = new URL(baseUrl);
		url.searchParams.set(name, value);
		return url.toString();
	},

	/**
	 * Gets the current page URL
	 */
	getCurrentUrl(): string {
		return window.location.href;
	},

	/**
	 * Gets the base URL (without search params)
	 */
	getBaseUrl(): string {
		return `${window.location.origin}${window.location.pathname}`;
	},

	/**
	 * Creates a message URL with base64-encoded message
	 */
	createMessageUrl(baseUrl: string, message: string): string {
		const encodedMessage = base64Utils.encodeString(message);
		if (!encodedMessage) {
			throw new Error('Failed to encode message');
		}
		return this.setParam(baseUrl, 'm', encodedMessage);
	}
};

// Import compact encoding at the top of the file when available
let compactEncodingModule: typeof import('./compactEncoding') | null = null;

// Gift assignment utilities
export const assignmentUtils = {
	/**
	 * Creates a gift assignment message payload
	 */
	createAssignmentPayload(giverName: string, receiverName: string): AssignmentPayload {
		return {
			giver: giverName,
			receiver: receiverName
		};
	},

	/**
	 * Creates a shareable assignment URL
	 * Uses new compact encoding if available, falls back to JSON
	 */
	async createAssignmentUrl(
		baseUrl: string,
		giverName: string,
		receiverName: string
	): Promise<string> {
		// Try to use compact encoding if available
		if (!compactEncodingModule) {
			try {
				compactEncodingModule = await import('./compactEncoding');
			} catch {
				// Compact encoding not available, fall back to JSON
			}
		}

		if (compactEncodingModule) {
			try {
				return compactEncodingModule.compactEncoding.createShareableUrl(
					baseUrl,
					giverName,
					receiverName
				);
			} catch (e) {
				devConsole.warn('Failed to use compact encoding, falling back to JSON:', e);
			}
		}

		// Fallback to JSON encoding
		const payload = this.createAssignmentPayload(giverName, receiverName);
		return versionedUrlUtils.createVersionedMessageUrl(baseUrl, payload);
	},

	/**
	 * Creates a shareable assignment URL (sync version for compatibility)
	 * Always uses JSON encoding
	 */
	createAssignmentUrlSync(baseUrl: string, giverName: string, receiverName: string): string {
		const payload = this.createAssignmentPayload(giverName, receiverName);
		return versionedUrlUtils.createVersionedMessageUrl(baseUrl, payload);
	}
};

// Input validation utilities
export const validationUtils = {
	/**
	 * Validates if a string is not empty after trimming
	 */
	isNonEmptyString(value: string): boolean {
		return typeof value === 'string' && value.trim().length > 0;
	},

	/**
	 * Validates if an array has at least the minimum required items
	 */
	hasMinimumItems<T>(array: T[], minItems: number): boolean {
		return Array.isArray(array) && array.length >= minItems;
	},

	/**
	 * Validates if two indices are different and not null
	 */
	areValidPairIndices(index1: number | null, index2: number | null): boolean {
		return index1 !== null && index2 !== null && index1 !== index2;
	}
};

// Error handling utilities
export const errorUtils = {
	/**
	 * Safely handles errors and returns a default value
	 */
	handleError<T>(operation: () => T, defaultValue: T, errorMessage?: string): T {
		try {
			return operation();
		} catch (error) {
			if (errorMessage) {
				devConsole.error(errorMessage, error);
			}
			return defaultValue;
		}
	},

	/**
	 * Creates a user-friendly error message
	 */
	createUserErrorMessage(error: string, context?: string): string {
		if (context) {
			return `${context}: ${error}`;
		}
		return error;
	}
};

// Versioning system for URL parameters
export const URL_PARAM_VERSION = 1;

export interface VersionedData<T = unknown> {
	version: number | string;
	data: T;
}

// Versioned URL utilities
export const versionedUrlUtils = {
	/**
	 * Creates versioned data structure
	 */
	createVersionedData<T>(data: T, version: number = URL_PARAM_VERSION): VersionedData<T> {
		return { version, data };
	},

	/**
	 * Migrates old data format to current version
	 */
	migrateData(versionedData: unknown): unknown {
		// If it's already versioned, return as-is
		if (
			typeof versionedData === 'object' &&
			versionedData !== null &&
			'version' in versionedData &&
			'data' in versionedData
		) {
			const versioned = versionedData as VersionedData;

			// Handle different versions here
			switch (versioned.version) {
				case 1:
				case '1.0': // Backward compatibility with string version
					return versioned.data;
				default:
					// Unknown version, return as-is and hope for the best
					return versioned.data;
			}
		}

		// Legacy data (no version), assume it's compatible
		return versionedData;
	},

	/**
	 * Creates a versioned message URL
	 */
	createVersionedMessageUrl(baseUrl: string, payload: AssignmentPayload): string {
		const versionedPayload = this.createVersionedData(payload);
		const encodedPayload = base64Utils.encodeJson(versionedPayload);
		if (!encodedPayload) {
			throw new Error('Failed to encode versioned message payload');
		}
		return urlUtils.setParam(baseUrl, 'm', encodedPayload);
	},

	/**
	 * Creates a versioned state URL
	 */
	createVersionedStateUrl(baseUrl: string, state: AppState): string {
		const versionedState = this.createVersionedData(state);
		const encodedState = base64Utils.encodeJson(versionedState);
		if (!encodedState) {
			throw new Error('Failed to encode versioned state');
		}
		return urlUtils.setParam(baseUrl, 's', encodedState);
	},

	/**
	 * Decodes versioned message from URL parameter
	 */
	decodeVersionedMessage(encodedParam: string): AssignmentPayload | null {
		const decoded = base64Utils.decodeJson(encodedParam);
		if (!decoded) return null;

		const migrated = this.migrateData(decoded);

		// Handle new payload format
		if (
			typeof migrated === 'object' &&
			migrated !== null &&
			'giver' in migrated &&
			'receiver' in migrated
		) {
			return migrated as AssignmentPayload;
		}

		// Handle legacy string format - convert to new format
		if (typeof migrated === 'string') {
			// Try to parse legacy format: "GiverName you are giving to ReceiverName"
			const legacyMatch = migrated.match(/^(.+?)\s+you are giving to\s+(.+)$/);
			if (legacyMatch) {
				return {
					giver: legacyMatch[1].trim(),
					receiver: legacyMatch[2].trim()
				};
			}
		}

		return null;
	},

	/**
	 * Decodes versioned state from URL parameter
	 */
	decodeVersionedState(encodedParam: string): Partial<AppState> | null {
		const decoded = base64Utils.decodeJson(encodedParam);
		if (!decoded) return null;

		const migrated = this.migrateData(decoded);

		// Handle both versioned and legacy formats
		if (typeof migrated === 'object' && migrated !== null) {
			return migrated as Partial<AppState>;
		}

		return null;
	}
};

// Loading state management
export interface LoadingState {
	isLoading: boolean;
	message?: string;
	error?: string;
}

export const loadingUtils = {
	/**
	 * Creates a loading state
	 */
	createLoading(message?: string): LoadingState {
		return { isLoading: true, message };
	},

	/**
	 * Creates a success state
	 */
	createSuccess(): LoadingState {
		return { isLoading: false };
	},

	/**
	 * Creates an error state
	 */
	createError(error: string): LoadingState {
		return { isLoading: false, error };
	}
};

// Type definitions for better TypeScript support
export interface AssignmentPayload {
	giver: string;
	receiver: string;
}

export interface AssignmentPair {
	giver: string;
	receiver: string;
	url: string;
}

export interface ForbiddenPair {
	index1: number;
	index2: number;
}

export interface AppState {
	names: string[];
	selection: number[];
	forbiddenPairs: ForbiddenPair[];
	assignmentHistory: number[][];
}
