import type { WorkoutDef } from './types'
import { g, deg } from './utils'

export const squat: WorkoutDef = {
	id: 'squat',
	name: 'Squats',
	cue: 'Stand side-on to the camera so your knee angle is visible.',
	init: () => ({ phase: 'up', count: 0 }),
	detect(pose, st) {
		let count = st.count as number
		let phase = st.phase as string

		const lh = g(pose, 'left_hip'), lk = g(pose, 'left_knee'), la = g(pose, 'left_ankle')
		const rh = g(pose, 'right_hip'), rk = g(pose, 'right_knee'), ra = g(pose, 'right_ankle')
		const pts = lh && lk && la ? [lh, lk, la] : rh && rk && ra ? [rh, rk, ra] : null
		if (!pts) return { count, phase, st }

		const a = deg(pts[0], pts[1], pts[2])
		if (a < 90 && phase === 'up') phase = 'down'
		if (a > 160 && phase === 'down') { 
		    phase = 'up'; 
		    count++ 
		}

		return { count, phase, st: { count, phase } }
	}
}
