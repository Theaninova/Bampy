import { Plane, Line3, Vector3 } from 'three';
import type { SlicerOptions } from '../slicer';
import { approxEquals } from '../util/equals';

/**
 * A continuous 2d ring of points
 *
 * No matter how you slice, on a model without holes all points will
 * form continous rings.
 */
export interface ShellRing {
	/**
	 * The plane the ring is on
	 */
	plane: Plane;

	/**
	 * Points of the ring
	 */
	points: Vector3[];
}

interface BaseSlice {
	/**
	 * The plane the slice is on
	 */
	plane: Plane;

	/**
	 * The lines of the slice (not sorted!)
	 */
	lines: Line3[];
}

const line = new Line3();
function intersect(layerPlane: Plane, a: Vector3, b: Vector3, targetVector: Vector3) {
	line.set(a, b);
	return layerPlane.intersectLine(line, targetVector);
}

/**
 * Creates base slices from the geometry, excluding surfaces.
 *
 * The slicse are not sorted or separated into rings.
 */
function createBaseSlices(options: SlicerOptions, surfaceTriangles: boolean[]): BaseSlice[] {
	const targetVector1 = new Vector3();
	const targetVector2 = new Vector3();
	const targetVector3 = new Vector3();

	const baseSlices: BaseSlice[] = [];

	for (let layer = 0; layer < options.geometry.boundingBox!.max.z; layer += options.layerHeight) {
		const baseSlice: BaseSlice = {
			plane: new Plane(options.bedNormal, -layer),
			lines: []
		};

		options.bvh.shapecast({
			intersectsBounds(box, _isLeaf, _score, _depth, _nodeIndex) {
				return baseSlice.plane.intersectsBox(box);
			},
			intersectsTriangle(target, triangleIndex, _contained, _depth) {
				if (surfaceTriangles[triangleIndex]) return;
				const intersections = [
					intersect(baseSlice.plane, target.a, target.b, targetVector1),
					intersect(baseSlice.plane, target.b, target.c, targetVector2),
					intersect(baseSlice.plane, target.c, target.a, targetVector3)
				];

				for (let i = 0; i < 3; i++) {
					const a = intersections[i];
					const b = intersections[(i + 1) % 3];
					if (a === null || b === null) continue;
					baseSlice.lines.push(new Line3(a.clone(), b.clone()));
					return;
				}
			}
		});

		baseSlices.push(baseSlice);
	}

	return baseSlices;
}

/**
 * Creates shell rings from the base slice
 *
 * Consumes the base slices
 */
function createShellRings(baseSlice: BaseSlice): ShellRing[] {
	const shellRings: ShellRing[] = [];

	let left: Vector3;
	let right: Vector3;

	while (baseSlice.lines.length > 0) {
		const start = baseSlice.lines.pop()!;
		const shellRing: ShellRing = {
			plane: baseSlice.plane,
			points: [start.start, start.end]
		};
		left = shellRing.points[0];
		right = shellRing.points[1];

		// This should use a linked list ideally, but whatever
		while (!approxEquals(left, right)) {
			for (let i = 0; i < baseSlice.lines.length; i++) {
				const line = baseSlice.lines[i];
				if (approxEquals(line.start, right)) {
					shellRing.points.push(line.start, line.end);
					right = line.end;
					baseSlice.lines.splice(i, 1);
					break;
				} else if (approxEquals(line.end, right)) {
					shellRing.points.push(line.end, line.start);
					right = line.start;
					baseSlice.lines.splice(i, 1);
					break;
				}
			}
		}

		shellRings.push(shellRing);
	}
	return shellRings;
}

/**
 * Creates standard slices from the geometry, excluding surfaces.
 */
export function slice(options: SlicerOptions, surfaceTriangles: boolean[]) {
	const shellRings = createBaseSlices(options, surfaceTriangles).map(createShellRings);
}
