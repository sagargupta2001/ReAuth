/**
 * True when the user has asked the OS to reduce motion.
 *
 * Every ambient or looping animation must check this — a continuously moving
 * backdrop is exactly what the setting exists to suppress.
 */
export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}
