import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { ConfirmDialog } from './confirm-dialog'

describe('ConfirmDialog', () => {
  it('renders correctly and confirms', () => {
    const onOpenChange = vi.fn()
    const handleConfirm = vi.fn()
    
    render(
      <ConfirmDialog
        open={true}
        onOpenChange={onOpenChange}
        title="Confirm Delete"
        desc="Are you really sure?"
        handleConfirm={handleConfirm}
      />
    )
    
    expect(screen.getByText('Confirm Delete')).toBeInTheDocument()
    expect(screen.getByText('Are you really sure?')).toBeInTheDocument()
    
    fireEvent.click(screen.getByText('Continue'))
    expect(handleConfirm).toHaveBeenCalledTimes(1)
  })
})
