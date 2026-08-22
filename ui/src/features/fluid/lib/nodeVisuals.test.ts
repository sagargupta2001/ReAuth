import { describe, expect, it } from 'vitest'

import {
  TEXT_FALLBACK,
  computeNodeVisuals,
  resolveDisplayText,
  resolveRadius,
  resolveVisibleFlag,
} from './nodeVisuals'
import type { ThemeNode } from '@/entities/theme/model/types'

function node(overrides: Partial<ThemeNode> = {}): ThemeNode {
  return { id: 'n1', type: 'Text', ...overrides }
}

describe('computeNodeVisuals: spacing', () => {
  it('omits spacing the node does not ask for', () => {
    // Emitting `0px` inline beat the container's space-y-* and flattened the page.
    const { style } = computeNodeVisuals(node())
    expect(style.marginTop).toBeUndefined()
    expect(style.marginBottom).toBeUndefined()
    expect(style.padding).toBeUndefined()
  })

  it('emits spacing the node sets', () => {
    const { style } = computeNodeVisuals(
      node({ props: { margin_top: 24, margin_bottom: 8, padding: 4 } }),
    )
    expect(style).toMatchObject({
      marginTop: '24px',
      marginBottom: '8px',
      padding: '4px',
    })
  })

  it('treats an explicit zero as absent', () => {
    const { style } = computeNodeVisuals(node({ props: { margin_top: 0 } }))
    expect(style.marginTop).toBeUndefined()
  })
})

describe('computeNodeVisuals: sizing', () => {
  it('defaults to fill width and hug height', () => {
    const visuals = computeNodeVisuals(node())
    expect(visuals.widthClass).toBe('w-full')
    expect(visuals.heightClass).toBe('h-auto')
    expect(visuals.fillWidthClass).toBe('w-full')
  })

  it('hugs when asked', () => {
    const visuals = computeNodeVisuals(node({ size: { width: 'hug', height: 'hug' } }))
    expect(visuals.widthClass).toBe('w-auto')
    expect(visuals.fillWidthClass).toBe('')
  })

  it('applies explicit dimensions only in fixed mode', () => {
    const fixed = computeNodeVisuals(
      node({ size: { width: 'fixed', width_value: '200px', height: 'fixed', height_value: '40px' } }),
    )
    expect(fixed.style.width).toBe('200px')
    expect(fixed.style.height).toBe('40px')

    const filling = computeNodeVisuals(node({ size: { width: 'fill', height: 'fill' } }))
    expect(filling.style.width).toBeUndefined()
    expect(filling.heightClass).toBe('h-full')
  })

  it('prefers node.size over legacy props', () => {
    const visuals = computeNodeVisuals(
      node({ size: { width: 'hug' }, props: { width: 'fill' } }),
    )
    expect(visuals.widthClass).toBe('w-auto')
  })

  it('maps the size prop to a height and text class', () => {
    expect(computeNodeVisuals(node({ props: { size: 'sm' } })).sizeClass).toBe('h-8 text-xs')
    expect(computeNodeVisuals(node({ props: { size: 'lg' } })).sizeClass).toBe('h-11 text-base')
    expect(computeNodeVisuals(node()).sizeClass).toBe('h-9 text-sm')
  })
})

describe('computeNodeVisuals: typography', () => {
  it('passes a numeric font weight through as a number', () => {
    expect(computeNodeVisuals(node({ props: { font_weight: '600' } })).style.fontWeight).toBe(600)
  })

  it('keeps a keyword font weight as a string', () => {
    expect(computeNodeVisuals(node({ props: { font_weight: 'bold' } })).style.fontWeight).toBe(
      'bold',
    )
  })

  it('reports raw typography props so callers can detect defaults', () => {
    const set = computeNodeVisuals(node({ props: { font_size: '12px' } }))
    expect(set.fontSize).toBe('12px')
    expect(computeNodeVisuals(node()).fontSize).toBe('')
  })

  it('maps alignment to a text class', () => {
    expect(computeNodeVisuals(node({ props: { align: 'center' } })).alignClass).toBe('text-center')
    expect(computeNodeVisuals(node({ props: { align: 'right' } })).alignClass).toBe('text-right')
    expect(computeNodeVisuals(node()).alignClass).toBe('text-left')
  })
})

describe('resolveDisplayText', () => {
  it('prefers literal copy', () => {
    expect(resolveDisplayText({ text: 'Welcome', text_path: 'message' })).toEqual({
      text: 'Welcome',
      isBinding: false,
    })
  })

  it('surfaces the binding when only a path is set', () => {
    expect(resolveDisplayText({ text_path: 'message' })).toEqual({
      text: '{message}',
      isBinding: true,
    })
  })

  it('falls back when neither is set', () => {
    expect(resolveDisplayText({})).toEqual({ text: TEXT_FALLBACK, isBinding: false })
  })
})

describe('resolveVisibleFlag', () => {
  it('defaults to visible', () => {
    expect(resolveVisibleFlag(undefined)).toBe(true)
  })

  it('honours booleans and the string "false"', () => {
    expect(resolveVisibleFlag(false)).toBe(false)
    expect(resolveVisibleFlag('false')).toBe(false)
    expect(resolveVisibleFlag('FALSE')).toBe(false)
    expect(resolveVisibleFlag('true')).toBe(true)
  })

  it('coerces numbers, so 0 hides', () => {
    // The builder used to return true here, disagreeing with the runtime.
    expect(resolveVisibleFlag(0)).toBe(false)
    expect(resolveVisibleFlag(2)).toBe(true)
  })
})

describe('resolveRadius', () => {
  it('adds px to a unitless value', () => {
    expect(resolveRadius(12)).toBe('12px')
    expect(resolveRadius('8')).toBe('8px')
    expect(resolveRadius(0.5)).toBe('0.5px')
  })

  it('leaves a value that already has a unit alone', () => {
    expect(resolveRadius('50%')).toBe('50%')
    expect(resolveRadius('1rem')).toBe('1rem')
  })

  it('returns empty for a missing value', () => {
    expect(resolveRadius(undefined)).toBe('')
    expect(resolveRadius('')).toBe('')
  })
})
