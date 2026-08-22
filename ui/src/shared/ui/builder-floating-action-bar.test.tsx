import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'

import { BuilderFloatingActionBar } from './builder-floating-action-bar'

describe('BuilderFloatingActionBar', () => {
  it('wires undo, redo, and save actions', () => {
    const onUndo = vi.fn()
    const onRedo = vi.fn()
    const onSave = vi.fn()

    render(
      <BuilderFloatingActionBar
        canUndo
        canRedo
        onUndo={onUndo}
        onRedo={onRedo}
        onSave={onSave}
      />,
    )

    fireEvent.click(screen.getByLabelText('Undo'))
    expect(onUndo).toHaveBeenCalledTimes(1)

    fireEvent.click(screen.getByLabelText('Redo'))
    expect(onRedo).toHaveBeenCalledTimes(1)

    fireEvent.click(screen.getByText('Save Draft'))
    expect(onSave).toHaveBeenCalledTimes(1)
  })

  it('disables history controls when there is nothing to undo or redo', () => {
    render(
      <BuilderFloatingActionBar
        canUndo={false}
        canRedo={false}
        onUndo={vi.fn()}
        onRedo={vi.fn()}
        onSave={vi.fn()}
      />,
    )

    expect(screen.getByLabelText('Undo')).toBeDisabled()
    expect(screen.getByLabelText('Redo')).toBeDisabled()
  })

  it('renders the inspect toggle only when a handler is provided', () => {
    const onToggleInspect = vi.fn()

    const { rerender } = render(
      <BuilderFloatingActionBar
        canUndo
        canRedo
        onUndo={vi.fn()}
        onRedo={vi.fn()}
        onSave={vi.fn()}
      />,
    )
    expect(screen.queryByLabelText('Toggle inspect')).not.toBeInTheDocument()

    rerender(
      <BuilderFloatingActionBar
        canUndo
        canRedo
        onUndo={vi.fn()}
        onRedo={vi.fn()}
        onSave={vi.fn()}
        isInspecting={false}
        onToggleInspect={onToggleInspect}
      />,
    )
    fireEvent.click(screen.getByLabelText('Toggle inspect'))
    expect(onToggleInspect).toHaveBeenCalledTimes(1)
  })
})
