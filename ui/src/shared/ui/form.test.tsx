import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { useForm } from 'react-hook-form'
import { useEffect } from 'react'
import { Form, FormField, FormItem, FormLabel, FormControl, FormDescription, FormMessage } from './form'

const TestForm = ({ error }: { error?: string }) => {
  const form = useForm({ defaultValues: { test: '' } })
  
  const { setError } = form;
  useEffect(() => {
    if (error) {
      setError('test', { message: error })
    }
  }, [error, setError])

  return (
    <Form {...form}>
      <FormField
        control={form.control}
        name="test"
        render={() => (
          <FormItem>
            <FormLabel>Label</FormLabel>
            <FormControl>
              <input />
            </FormControl>
            <FormDescription>Description</FormDescription>
            <FormMessage />
          </FormItem>
        )}
      />
    </Form>
  )
}

describe('Form Components', () => {
  it('renders correctly with label and description', () => {
    render(<TestForm />)
    expect(screen.getByText('Label')).toBeInTheDocument()
    expect(screen.getByText('Description')).toBeInTheDocument()
  })

  it('displays error message when field has error', () => {
    render(<TestForm error="Test error" />)
    expect(screen.getByText('Test error')).toBeInTheDocument()
    expect(screen.getByText('Label')).toHaveClass('text-destructive')
  })
})
