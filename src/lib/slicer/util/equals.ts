import type { Vector3 } from 'three';

/**
 * Check if two vectors are approximately equal.
 */
export function approxEquals(a: Vector3, b: Vector3, epsilon = Number.EPSILON) {
	return (
		Math.abs(a.x - b.x) < epsilon && Math.abs(a.y - b.y) < epsilon && Math.abs(a.z - b.z) < epsilon
	);
}
