import type { Vector3Tuple } from 'three';

export interface SliceArguments {
	stl: object;
	bedNormal: Vector3Tuple;
	maxNonPlanarAngle: number;
	tolerance: number;
	layerHeight: number;
	nozzleDiameter: number;
	minSurfacePathLength: number;
}

export interface SliceEvent {
	type: 'slice';
	data: SliceArguments;
}

export type WorkerEvent = SliceEvent;

export type WorkerMessage = LayerMessage | ProgressMessage;

export interface LayerMessage {
	type: 'layer';
	data: Layer;
}

export interface ProgressMessage {
	type: 'progress';
	percent?: number;
	layer: number;
}

export interface Layer {
	type: LayerType;
	geometry: object;
}

export const enum LayerType {
	Line,
	Surface
}
