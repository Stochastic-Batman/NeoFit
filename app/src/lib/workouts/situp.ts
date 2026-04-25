import type { WorkoutDef } from './types'
import { g, deg } from './utils'


export const situp: WorkoutDef = {
	id: 'situp',
	name: 'Sit-Ups',
	cue: 'Lie sideways to the camera so your hip and knee are visible. Knees bent.',
	init: () => ({ phase: 'down', count: 0 }),
	detect(pose, st) {
		let count = st.count as number
		let phase = st.phase as string

		const ls = g(pose, 'left_shoulder'), lh = g(pose, 'left_hip'), lk = g(pose, 'left_knee')
		const rs = g(pose, 'right_shoulder'), rh = g(pose, 'right_hip'), rk = g(pose, 'right_knee')
		const pts = ls && lh && lk ? [ls, lh, lk] : rs && rh && rk ? [rs, rh, rk] : null
		if (!pts) return { count, phase, st }

		const a = deg(pts[0], pts[1], pts[2])
		if (a < 75 && phase === 'down') phase = 'up'
		if (a > 140 && phase === 'up') { phase = 'down'; count++ }

		return { count, phase, st: { count, phase } }
	}
}
