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

export type ExerciseDef = {
	id: number    // the value sent in log_reps
	slug: string  // matches the WorkoutDef.id string used inside the app
	name: string
	cue: string
}

export interface WorkoutDef {
	id: string         // matches ExerciseDef.slug
	onChainId: number  // matches ExerciseDef.id; sent to the Anchor program
	name: string
	cue: string
	countLabel?: string
	params?: ParamDef[]
	init: (cfg?: Record<string, number>) => St
	detect: (pose: Pose, st: St) => { count: number; phase: string; st: St }
}
