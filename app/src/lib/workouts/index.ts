// app/src/lib/workouts/index.ts
export type { WorkoutDef, KP, Pose, St, ParamDef, ExerciseDef } from './types'
import type { WorkoutDef, ExerciseDef } from './types'

// exerciseList is the array from exercises.json at the workspace root.
// TypeScript infers its type from the JSON; we cast to ExerciseDef[] for
// clarity and to catch any future schema drift at compile time.
import exerciseList from '$exercises'

import { squat }       from './squat'
import { pushup }      from './pushup'
import { pullup }      from './pullup'
import { situp }       from './situp'
import { jumpingJack } from './jumpingJack'

// Detectors are keyed by slug so the lookup below is order-independent.
// Add new detector files here when exercises.json grows.
const detectors: Record<string, Omit<WorkoutDef, 'onChainId'>> = {
	squat,
	pushup,
	pullup,
	situp,
	jumping_jack: jumpingJack
}

// Build the canonical workouts array from the JSON definitions.
// name and cue come from exercises.json; detection logic comes from the
// individual detector files. The array order follows exercises.json order,
// which must match the on-chain exercise_id values (0 = squat, 1 = pushup …).
export const workouts: WorkoutDef[] = (exerciseList as ExerciseDef[]).map((e) => {
	const detector = detectors[e.slug]
	if (!detector) {
		throw new Error(
			`[workouts] No detector found for slug "${e.slug}". ` +
			`Add an entry to the detectors map in index.ts.`
		)
	}
	return {
		...detector,
		id:         e.slug,   // string key used by the UI (selId, routing, etc.)
		onChainId:  e.id,     // u8 sent in the log_reps instruction
		name:       e.name,
		cue:        e.cue
	}
})
