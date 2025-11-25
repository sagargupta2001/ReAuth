import type { AnimationEngine } from '@/lib/animations/animation.types.ts'
import { gsapAnimationEngine } from '@/lib/animations/gsap.engine.ts'

export const AnimationLibrary = {
  GSAP: 'gsap',
  FRAMER: 'framer',
} as const

export type AnimationLibrary = (typeof AnimationLibrary)[keyof typeof AnimationLibrary]

let CURRENT_ENGINE: AnimationEngine = gsapAnimationEngine

export function setAnimationLibrary(lib: AnimationLibrary) {
  switch (lib) {
    case AnimationLibrary.GSAP:
      CURRENT_ENGINE = gsapAnimationEngine
      break
  }
}

export function getAnimationEngine(): AnimationEngine {
  return CURRENT_ENGINE
}
