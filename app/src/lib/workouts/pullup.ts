import type { WorkoutDef } from './types'
import { g, deg } from './utils'

export const pullup: WorkoutDef = {
	id: 'pullup',
	name: 'Pull-Ups',
	cue: 'Face the camera with your full upper body visible while hanging from the bar.',
	init: () => ({ phase: 'down', count: 0 }),
	detect(pose, st) {
		let count = st.count as number
		let phase = st.phase as string

		const ls = g(pose, 'left_shoulder'),  le = g(pose, 'left_elbow'),  lw = g(pose, 'left_wrist')
		const rs = g(pose, 'right_shoulder'), re = g(pose, 'right_elbow'), rw = g(pose, 'right_wrist')
		const pts = ls && le && lw ? [ls, le, lw] : rs && re && rw ? [rs, re, rw] : null
		if (!pts) return { count, phase, st }

		const [shoulder, , wrist] = pts
		// Must be hanging (wrist above shoulder in image coords = lower y value)
		if (wrist.y >= shoulder.y) return { count, phase, st }

		const a = deg(pts[0], pts[1], pts[2])
		if (a < 70  && phase === 'down') phase = 'up'
		if (a > 155 && phase === 'up')   { phase = 'down'; count++ }

		return { count, phase, st: { count, phase } }
	}
}
