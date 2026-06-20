export interface AnimationEngine {
  fadeSlideIn: (el: HTMLElement, options?: AnimationOptions) => void
  fadeSlideOut: (el: HTMLElement, options?: AnimationOptions) => Promise<void>
  highlight: (el: HTMLElement, options?: HighlightOptions) => void
  /**
   * Fluidly morphs an element from one box size to another (iOS "Dynamic Island"
   * style). Inline sizing is cleared on completion so the element returns to its
   * natural, content-driven layout.
   */
  morphSize: (el: HTMLElement, options: MorphOptions) => Promise<void>
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

export interface BoxSize {
  width: number
  height: number
}

export interface MorphOptions {
  from: BoxSize
  to: BoxSize
  duration?: number
  ease?: string
  /** When true, applies a springy overshoot for a pronounced "pop". */
  overshoot?: boolean
}
