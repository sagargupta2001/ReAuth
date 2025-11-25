import gsap from 'gsap'

import type { AnimationEngine, AnimationOptions } from './animation.types'

export const gsapAnimationEngine: AnimationEngine = {
  fadeSlideIn(el, options = {}) {
    return new Promise<void>((resolve) => {
      const { y = 100, duration = 0.35, ease = 'power3.out', delay = 0 } = options

      gsap.fromTo(
        el,
        { y, opacity: 0 },
        {
          y: 0,
          opacity: 1,
          duration,
          ease,
          delay,
          onComplete: resolve,
        },
      )
    })
  },

  fadeSlideOut: (el: HTMLElement, options?: AnimationOptions): Promise<void> => {
    return new Promise((resolve) => {
      gsap.to(el, {
        opacity: 0,
        y: 20,
        duration: 0.3,
        onComplete: resolve, // resolves void
        ...options,
      })
    })
  },
}
