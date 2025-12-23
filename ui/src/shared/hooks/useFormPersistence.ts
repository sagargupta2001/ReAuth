import { useCallback, useEffect, useRef } from 'react'

import type { UseFormReturn } from 'react-hook-form'

import { useUnsavedChanges } from '@/shared/context/UnsavedChangesContext'

export function useFormPersistence<T extends import('react-hook-form').FieldValues>(
  form: UseFormReturn<T>,
  onSubmit: (data: T) => void,
  isPending: boolean,
) {
  const { registerForm, unregisterForm } = useUnsavedChanges()
  const { isDirty } = form.formState

  // 1. Store the latest 'onSubmit' in a ref.
  // This allows us to roles the *current* function inside the callback
  // without adding it to the dependency array.
  const onSubmitRef = useRef(onSubmit)
  useEffect(() => {
    onSubmitRef.current = onSubmit
  }, [onSubmit])

  // 2. Create stable handlers that don't change on every render
  const handleSubmit = useCallback(() => {
    // Always use the latest version of the submit handler
    form.handleSubmit(onSubmitRef.current)()
  }, [form])

  const handleReset = useCallback(() => {
    form.reset()
  }, [form])

  // 3. Register/Unregister Effect
  // Notice: 'onSubmit' is NOT in the dependency array anymore.
  useEffect(() => {
    if (isDirty) {
      registerForm({
        submit: handleSubmit,
        reset: handleReset,
        isPending,
      })
    } else {
      unregisterForm()
    }

    return () => {
      unregisterForm()
    }
  }, [isDirty, isPending, registerForm, unregisterForm, handleSubmit, handleReset])
}
