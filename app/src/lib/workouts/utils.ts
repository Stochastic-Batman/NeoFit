import type { KP, Pose } from './types'

export const THRESH = 0.25

export const g = (pose: Pose, name: string): KP | null =>
	pose.keypoints.find((k) => k.name === name && k.confidence > THRESH) ?? null

export const deg = (a: KP, b: KP, c: KP): number => {
	const [ax, ay] = [a.x - b.x, a.y - b.y]
	const [cx, cy] = [c.x - b.x, c.y - b.y]
	const d = ax * cx + ay * cy
	const m = Math.hypot(ax, ay) * Math.hypot(cx, cy)
	return m ? Math.acos(Math.max(-1, Math.min(1, d / m))) * (180 / Math.PI) : 0
}
