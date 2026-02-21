import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { AutoForm } from './auto-form'

describe('AutoForm', () => {
  const schema = {
    properties: {
      name: { title: 'Name', type: 'string', default: '' },
      enabled: { title: 'Enabled', type: 'boolean', default: true },
      count: { title: 'Count', type: 'integer', default: 0 },
    }
  }

  it('renders all field types correctly', () => {
    const handleChange = vi.fn()
    render(<AutoForm schema={schema} values={{}} onChange={handleChange} />)
    
    expect(screen.getByLabelText('Name')).toBeInTheDocument()
    expect(screen.getByLabelText('Enabled')).toBeInTheDocument()
    expect(screen.getByLabelText('Count')).toBeInTheDocument()
  })

  it('calls onChange when values change', () => {
    const handleChange = vi.fn()
    render(<AutoForm schema={schema} values={{ name: 'old' }} onChange={handleChange} />)
    
    const input = screen.getByLabelText('Name')
    fireEvent.change(input, { target: { value: 'new' } })
    
    expect(handleChange).toHaveBeenCalledWith(expect.objectContaining({ name: 'new' }))
  })
})
