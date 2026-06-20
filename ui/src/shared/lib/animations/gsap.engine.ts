import gsap from 'gsap'

import type {
  AnimationEngine,
  AnimationOptions,
  HighlightOptions,
  MorphOptions,
} from './animation.types'

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

  highlight: (el: HTMLElement, options: HighlightOptions = {}) => {
    const { duration = 1.6, hold = 0.2, ease = 'power2.out', peak = 1 } = options
    const half = Math.max(0.2, duration / 2)

    gsap.killTweensOf(el)
    gsap.set(el, { '--highlight-alpha': 0 })
    gsap.to(el, {
      '--highlight-alpha': peak,
      duration: half,
      ease,
      yoyo: true,
      repeat: 1,
      repeatDelay: hold,
      onComplete: () => {
        gsap.set(el, { '--highlight-alpha': 0 })
      },
    })
  },

  morphSize: (el: HTMLElement, options: MorphOptions): Promise<void> => {
    return new Promise((resolve) => {
      const { from, to, duration = 0.45, overshoot = true } = options
      // A springy overshoot gives the "Dynamic Island" pop; power3 stays smooth.
      const ease = options.ease ?? (overshoot ? 'back.out(1.7)' : 'power3.out')

      gsap.killTweensOf(el)

      const finish = () => {
        // Hand sizing back to the content so layout stays fluid/responsive.
        gsap.set(el, { width: 'auto', height: 'auto' })
        resolve()
      }

      // Nothing meaningful to animate — snap and bail.
      if (Math.round(from.width) === Math.round(to.width) &&
          Math.round(from.height) === Math.round(to.height)) {
        finish()
        return
      }

      gsap.fromTo(
        el,
        { width: from.width, height: from.height },
        {
          width: to.width,
          height: to.height,
          duration,
          ease,
          onComplete: finish,
        },
      )
    })
  },
}
