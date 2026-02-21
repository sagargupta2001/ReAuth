import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { FloatingActionBar } from './floating-action-bar'

describe('FloatingActionBar', () => {
  it('renders and calls actions', () => {
    const onSave = vi.fn()
    const onReset = vi.fn()
    
    render(
      <FloatingActionBar
        isOpen={true}
        onSave={onSave}
        onReset={onReset}
      />
    )
    
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument()
    
    fireEvent.click(screen.getByText('Save Changes'))
    expect(onSave).toHaveBeenCalledTimes(1)
    
    fireEvent.click(screen.getByText('Reset'))
    expect(onReset).toHaveBeenCalledTimes(1)
  })
})
