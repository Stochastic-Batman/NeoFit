import type { WorkoutDef } from './types'
import { g } from './utils'

export const jumpingJack: WorkoutDef = {
	id: 'jumping_jack',
	name: 'Jumping Jacks',
	cue: 'Face the camera straight on with your full body visible.',
	init: () => ({ phase: 'down', count: 0 }),
	detect(pose, st) {
		let count = st.count as number
		let phase = st.phase as string

		const lw = g(pose, 'left_wrist'), rw = g(pose, 'right_wrist')
		const ls = g(pose, 'left_shoulder'), rs = g(pose, 'right_shoulder')
		if (!lw || !rw || !ls || !rs) return { count, phase, st }

		const armsUp = lw.y < ls.y && rw.y < rs.y
		if (armsUp && phase === 'down') phase = 'up'
		if (!armsUp && phase === 'up') { phase = 'down'; count++ }

		return { count, phase, st: { count, phase } }
	}
}
