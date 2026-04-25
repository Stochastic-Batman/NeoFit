export type { WorkoutDef, KP, Pose, St } from './types'
import { squat } from './squat'
import { pushup } from './pushup'
import { jumpingJack } from './jumpingJack'
import { situp } from './situp'

export const workouts = [squat, pushup, jumpingJack, situp]
