import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { LocalPageLoader } from './local-page-loader'

describe('LocalPageLoader', () => {
  it('renders correctly with default message', () => {
    render(<LocalPageLoader />)
    expect(screen.getByText('Loading...')).toBeInTheDocument()
  })

  it('renders correctly with custom message', () => {
    render(<LocalPageLoader message="Please wait" />)
    expect(screen.getByText('Please wait')).toBeInTheDocument()
  })
})
