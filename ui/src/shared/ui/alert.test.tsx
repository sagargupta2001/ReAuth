import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Alert, AlertTitle, AlertDescription } from './alert'

describe('Alert', () => {
  it('renders correctly with default props', () => {
    render(
      <Alert>
        <AlertTitle>Title</AlertTitle>
        <AlertDescription>Description</AlertDescription>
      </Alert>
    )
    expect(screen.getByRole('alert')).toBeInTheDocument()
    expect(screen.getByText('Title')).toBeInTheDocument()
    expect(screen.getByText('Description')).toBeInTheDocument()
  })

  it('applies variant classes correctly', () => {
    render(<Alert variant="destructive">Error</Alert>)
    expect(screen.getByRole('alert')).toHaveClass('text-destructive')
  })
})
