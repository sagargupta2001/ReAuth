import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { PageErrorFallback } from './page-error-fallback'

describe('PageErrorFallback', () => {
  it('renders correctly and calls reset', () => {
    const error = new Error('Test Error')
    const reset = vi.fn()
    
    render(<PageErrorFallback error={error} resetErrorBoundary={reset} />)
    
    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
    expect(screen.getByText('Test Error')).toBeInTheDocument()
    
    fireEvent.click(screen.getByText('Try again'))
    expect(reset).toHaveBeenCalledTimes(1)
  })
})
