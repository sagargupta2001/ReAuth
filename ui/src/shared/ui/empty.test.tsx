import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Empty, EmptyHeader, EmptyTitle, EmptyDescription } from './empty'

describe('Empty', () => {
  it('renders correctly', () => {
    render(
      <Empty>
        <EmptyHeader>
          <EmptyTitle>No items</EmptyTitle>
          <EmptyDescription>Get started by creating one.</EmptyDescription>
        </EmptyHeader>
      </Empty>
    )
    
    expect(screen.getByText('No items')).toBeInTheDocument()
    expect(screen.getByText('Get started by creating one.')).toBeInTheDocument()
  })
})
