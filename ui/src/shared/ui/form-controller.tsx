import type { ReactNode } from 'react'

import { Controller, type ControllerProps, type FieldValues } from 'react-hook-form'

import { FormControl, FormDescription, FormItem, FormLabel, FormMessage } from '@/components/form'

interface FormControllerProps<T extends FieldValues> extends Omit<ControllerProps<T>, 'render'> {
  label?: string
  description?: string
  children: (field: any) => ReactNode
}

export function FormController<T extends FieldValues>({
  control,
  name,
  label,
  description,
  children,
  ...props
}: FormControllerProps<T>) {
  return (
    <Controller
      control={control}
      name={name}
      {...props}
      render={({ field }) => (
        <FormItem>
          {label && <FormLabel>{label}</FormLabel>}
          <FormControl>{children(field)}</FormControl>
          {description && <FormDescription>{description}</FormDescription>}
          <FormMessage />
        </FormItem>
      )}
    />
  )
}
