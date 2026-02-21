import { render } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Skeleton } from './skeleton'

describe('Skeleton', () => {
  it('renders correctly', () => {
    const { container } = render(<Skeleton className="h-4 w-4" />)
    expect(container.firstChild).toBeInTheDocument()
    expect(container.firstChild).toHaveClass('animate-pulse')
    expect(container.firstChild).toHaveClass('h-4 w-4')
  })
})
