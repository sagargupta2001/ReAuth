import { useEffect } from 'react'

import type { UseFormReturn } from 'react-hook-form'

import { useUnsavedChanges } from '@/shared/context/UnsavedChangesContext'

/**
 * Connects a React Hook Form instance to the Global Floating Action Bar.
 */
export function useFormPersistence<T extends import('react-hook-form').FieldValues>(
  form: UseFormReturn<T>,
  onSubmit: (data: T) => void,
  isPending: boolean,
) {
  const { registerForm, unregisterForm } = useUnsavedChanges()
  const { isDirty } = form.formState

  useEffect(() => {
    if (isDirty) {
      registerForm({
        submit: form.handleSubmit(onSubmit),
        reset: () => form.reset(), // Reset to default values
        isPending,
      })
    } else {
      unregisterForm()
    }

    // Cleanup on unmount
    return () => {
      // Only unregister if WE were the ones who registered (simple check)
      if (isDirty) unregisterForm()
    }
  }, [isDirty, isPending, form, registerForm, unregisterForm, onSubmit])
}
