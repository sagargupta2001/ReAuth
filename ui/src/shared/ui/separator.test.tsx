import { render } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Separator } from './separator'

describe('Separator', () => {
  it('renders correctly', () => {
    const { container } = render(<Separator />)
    expect(container.firstChild).toBeInTheDocument()
    expect(container.firstChild).toHaveClass('h-[1px] w-full')
  })

  it('renders correctly with vertical orientation', () => {
    const { container } = render(<Separator orientation="vertical" />)
    expect(container.firstChild).toHaveClass('h-full w-[1px]')
  })
})
