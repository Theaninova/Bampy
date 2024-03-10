import {
	BufferGeometry,
	BufferGeometryLoader,
	Float32BufferAttribute,
	Line3,
	MathUtils,
	Matrix4,
	Plane,
	Vector3
} from 'three';
import { ExtendedTriangle, MeshBVH, type HitPointInfo } from 'three-mesh-bvh';
import {
	LayerType,
	type LayerMessage,
	type SliceArguments,
	type ProgressMessage,
	type WorkerEvent
} from './worker-data';
import init, { slice } from 'bampy';

addEventListener('message', async (event: MessageEvent<WorkerEvent>) => {
	if (event.data.type === 'slice') {
		const geometry = new BufferGeometryLoader().parse(event.data.data.stl);
		if (geometry.index !== null) {
			geometry.toNonIndexed();
		}
		await init();
		slice(
			geometry.attributes.position.array as Float32Array,
			event.data.data.layerHeight,
			event.data.data.maxNonPlanarAngle
		);
	}
});

async function todo({
	stl,
	bedNormal: bedNormalArray,
	maxNonPlanarAngle,
	tolerance,
	layerHeight
}: SliceArguments) {
	greet();
	self.postMessage({ type: 'progress', percent: 0, layer: 0 } satisfies ProgressMessage);

	// TODO

	self.postMessage({
		type: 'progress',
		layer: Math.round(geometry.boundingBox!.max.z / layerHeight)
	} satisfies ProgressMessage);
}
