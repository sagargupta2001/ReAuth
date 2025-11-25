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

interface FormInputProps<T extends FieldValues> extends React.ComponentProps<typeof Input> {
  control: Control<T>
  name: Path<T>
  label?: string
  description?: string
}

export function FormInput<T extends FieldValues>({
  control,
  name,
  label,
  description,
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
            <Input {...props} {...field} />
          </FormControl>
          {description && <FormDescription>{description}</FormDescription>}
          <FormMessage />
        </FormItem>
      )}
    />
  )
}
