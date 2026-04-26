export type { WorkoutDef, KP, Pose, St, ParamDef } from './types'
import { squat } from './squat'
import { pushup } from './pushup'
import { pullup } from './pullup'
import { jumpingJack } from './jumpingJack'
import { situp } from './situp'

export const workouts = [squat, pushup, pullup, situp, jumpingJack]
