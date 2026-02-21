import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { useForm } from 'react-hook-form'
import { Form } from './form'
import { FormTextarea } from './form-textarea'

const TestForm = () => {
  const form = useForm({ defaultValues: { bio: '' } })
  return (
    <Form {...form}>
      <FormTextarea control={form.control} name="bio" label="Bio" placeholder="Enter bio" />
    </Form>
  )
}

describe('FormTextarea', () => {
  it('renders correctly with label and placeholder', () => {
    render(<TestForm />)
    expect(screen.getByLabelText('Bio')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Enter bio')).toBeInTheDocument()
  })
})
