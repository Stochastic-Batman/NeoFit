export type KP = { x: number; y: number; confidence: number; name: string }
export type Pose = { keypoints: KP[] }
export type St = Record<string, unknown>

export type ParamDef = {
	key: string
	label: string
	unit: string
	min: number
	max: number
	step: number
	default: number
}

export interface WorkoutDef {
	id: string
	name: string
	cue: string
	countLabel?: string
	params?: ParamDef[]
	init: (cfg?: Record<string, number>) => St
	detect: (pose: Pose, st: St) => { count: number; phase: string; st: St }
}
