import { type Control, type FieldValues, type Path } from 'react-hook-form'

import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/form'
import { Textarea } from '@/components/textarea'


interface FormTextareaProps<T extends FieldValues> extends React.ComponentProps<typeof Textarea> {
  control: Control<T>
  name: Path<T>
  label?: string
  description?: string
}

export function FormTextarea<T extends FieldValues>({
  control,
  name,
  label,
  description,
  ...props
}: FormTextareaProps<T>) {
  return (
    <FormField
      control={control}
      name={name}
      render={({ field }) => (
        <FormItem>
          {label && <FormLabel>{label}</FormLabel>}
          <FormControl>
            {/* Spread props first, then field to ensure RHF handlers take precedence */}
            <Textarea className="resize-none" {...props} {...field} />
          </FormControl>
          {description && <FormDescription>{description}</FormDescription>}
          <FormMessage />
        </FormItem>
      )}
    />
  )
}
