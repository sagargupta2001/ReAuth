export interface AnimationEngine {
  fadeSlideIn: (el: HTMLElement, options?: AnimationOptions) => void
  fadeSlideOut: (el: HTMLElement, options?: AnimationOptions) => Promise<void>
  highlight: (el: HTMLElement, options?: HighlightOptions) => void
}

export interface AnimationOptions {
  y?: number
  duration?: number
  delay?: number
  ease?: string
}

export interface HighlightOptions {
  duration?: number
  hold?: number
  ease?: string
  peak?: number
}
