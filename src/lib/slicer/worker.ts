import { BufferGeometry, BufferGeometryLoader, Float32BufferAttribute } from 'three';
import {
	type SliceArguments,
	type ProgressMessage,
	type WorkerEvent,
	type LayerMessage,
	LayerType
} from './worker-data';
import init, { slice } from 'bampy';

addEventListener('message', async (event: MessageEvent<WorkerEvent>) => {
	if (event.data.type === 'slice') {
		const geometry = new BufferGeometryLoader().parse(event.data.data.stl);
		if (geometry.index !== null) {
			geometry.toNonIndexed();
		}
		await init();
		const result = slice({
			positions: geometry.attributes.position.array as Float32Array,
			layerHeight: event.data.data.layerHeight,
			maxAngle: event.data.data.maxNonPlanarAngle,
			nozzleDiameter: event.data.data.nozzleDiameter,
			minSurfacePathLength: event.data.data.minSurfacePathLength
		});
		for (const layer of result.slices) {
			self.postMessage({
				type: 'layer',
				data: layer
			});
		}
	}
});
