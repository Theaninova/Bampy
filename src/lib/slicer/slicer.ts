import {
	BufferAttribute,
	BufferGeometry,
	BufferGeometryLoader,
	InterleavedBufferAttribute,
	Vector3
} from 'three';
import type { SliceArguments } from './worker-data';
import { MeshBVH } from 'three-mesh-bvh';

export class SlicerOptions {
	readonly bedNormal: Vector3;
	readonly maxNonPlanarAngle: number;
	readonly tolerance: number;
	readonly layerHeight: number;

	readonly geometry: BufferGeometry;
	readonly bvh: MeshBVH;
	readonly positions: BufferAttribute | InterleavedBufferAttribute;
	readonly normals: BufferAttribute | InterleavedBufferAttribute;
	readonly index: BufferAttribute;

	constructor(options: SliceArguments) {
		this.bedNormal = new Vector3(...options.bedNormal);
		this.maxNonPlanarAngle = options.maxNonPlanarAngle;
		this.tolerance = options.tolerance;
		this.layerHeight = options.layerHeight;

		this.geometry = new BufferGeometryLoader().parse(options.stl);
		this.bvh = new MeshBVH(this.geometry);
		this.positions = this.geometry.getAttribute('position');
		this.normals = this.geometry.getAttribute('normal');
		this.index = this.geometry.index!;
	}
}
