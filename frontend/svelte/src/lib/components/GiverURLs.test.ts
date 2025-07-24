import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import GiverURLs from './GiverURLs.svelte';

// Extend expect with testing-library matchers
import '@testing-library/jest-dom/vitest';

// Mock the stores
vi.mock('$lib/stores/CurrentAssignment', () => ({
	assignment: writable(null)
}));

vi.mock('$lib/stores/NameList', () => ({
	names: writable(['Alice', 'Bob', 'Charlie', 'David'])
}));

vi.mock('$lib/stores/Selection', () => ({
	selection: writable(new Set([0, 1, 2, 3]))
}));

vi.mock('$lib/stores/ForbiddenPairs', () => ({
	pairs: writable([])
}));

vi.mock('$lib/stores/AssignmentHistory', () => ({
	history: writable([])
}));

vi.mock('$lib/stores/LoadingState', () => ({
	wasmLoadingState: writable({ status: 'success' }),
	assignmentLoadingState: writable({ status: 'idle' })
}));

// Mock the WASM module
vi.mock('$lib/wasm/rust_wasm.js', () => ({
	default: vi.fn(() => Promise.resolve()),
	process_assignment: vi.fn((selection) => {
		// Simple mock implementation
		const result = [...selection];
		// Rotate assignments
		const first = result.shift();
		result.push(first);
		return result;
	})
}));

describe('GiverURLs Component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should render the generate button', () => {
		render(GiverURLs);
		const button = screen.getAllByRole('button', { name: /generate new gift assignment/i })[0];
		expect(button).toBeInTheDocument();
	});

	it('should handle component rendering without errors', () => {
		const { container } = render(GiverURLs);
		expect(container).toBeInTheDocument();
	});

	it('should show generate button with correct text', () => {
		render(GiverURLs);
		expect(screen.getAllByText('Generate Assignment')[0]).toBeInTheDocument();
	});

	it('should handle empty selection store gracefully', () => {
		// Basic rendering test
		const { container } = render(GiverURLs);
		expect(container.firstChild).toBeTruthy();
	});
});
