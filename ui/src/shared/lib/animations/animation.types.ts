export interface AnimationEngine {
  fadeSlideIn: (el: HTMLElement, options?: AnimationOptions) => void
  fadeSlideOut: (el: HTMLElement, options?: AnimationOptions) => Promise<void>
}

export interface AnimationOptions {
  y?: number
  duration?: number
  delay?: number
  ease?: string
}
