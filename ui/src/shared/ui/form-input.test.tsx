import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { useForm } from 'react-hook-form'
import { Form } from './form'
import { FormInput } from './form-input'

const TestForm = () => {
  const form = useForm({ defaultValues: { name: '' } })
  return (
    <Form {...form}>
      <FormInput control={form.control} name="name" label="Name" placeholder="Enter name" />
    </Form>
  )
}

describe('FormInput', () => {
  it('renders correctly with label and placeholder', () => {
    render(<TestForm />)
    expect(screen.getByLabelText('Name')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Enter name')).toBeInTheDocument()
  })
})
