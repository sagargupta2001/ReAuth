import * as React from 'react'
import { type Control, type FieldValues, type Path } from 'react-hook-form'

import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/form'
import { Input } from '@/components/input'

export interface FormInputProps<T extends FieldValues> extends React.ComponentProps<typeof Input> {
  control: Control<T>
  name: Path<T>
  label?: string
  description?: string
  render?: (input: React.ReactNode) => React.ReactNode
}

export function FormInput<T extends FieldValues>({
  control,
  name,
  label,
  description,
  render,
  ...props
}: FormInputProps<T>) {
  return (
    <FormField
      control={control}
      name={name}
      render={({ field }) => (
        <FormItem>
          {label && <FormLabel>{label}</FormLabel>}
          <FormControl>
            {/* We spread `props` first (user overrides),
              then `field` (react-hook-form logic).
            */}
            {render ? (
              render(<Input {...props} {...field} />)
            ) : (
              <Input {...props} {...field} />
            )}
          </FormControl>
          {description && <FormDescription>{description}</FormDescription>}
          <FormMessage />
        </FormItem>
      )}
    />
  )
}
