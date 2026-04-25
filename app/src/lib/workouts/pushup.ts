import type { WorkoutDef } from './types'
import { g, deg } from './utils'

export const pushup: WorkoutDef = {
	id: 'pushup',
	name: 'Push-Ups',
	cue: 'Get into a plank facing sideways so your elbow angle is visible.',
	init: () => ({ phase: 'up', count: 0 }),
	detect(pose, st) {
		let count = st.count as number
		let phase = st.phase as string

		const ls = g(pose, 'left_shoulder'), le = g(pose, 'left_elbow'), lw = g(pose, 'left_wrist')
		const rs = g(pose, 'right_shoulder'), re = g(pose, 'right_elbow'), rw = g(pose, 'right_wrist')
		const pts = ls && le && lw ? [ls, le, lw] : rs && re && rw ? [rs, re, rw] : null
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
