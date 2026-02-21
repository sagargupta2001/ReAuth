import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { ScrollArea } from './scroll-area'

describe('ScrollArea', () => {
  it('renders correctly', () => {
    render(
      <ScrollArea className="h-20 w-20">
        <div>Scroll content</div>
      </ScrollArea>
    )
    expect(screen.getByText('Scroll content')).toBeInTheDocument()
  })
})
