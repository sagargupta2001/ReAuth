import { useCallback, useEffect, useRef } from 'react'

import type { UseFormReturn } from 'react-hook-form'

import { useUnsavedChanges } from '@/shared/context/UnsavedChangesContext'

interface PersistenceOptions {
  enabled?: boolean
}

export function useFormPersistence<T extends import('react-hook-form').FieldValues>(
  form: UseFormReturn<T>,
  onSubmit: (data: T) => void,
  isPending: boolean,
  options: PersistenceOptions = {},
) {
  const { registerForm, unregisterForm } = useUnsavedChanges()
  const { isDirty } = form.formState

  const isEnabled = options.enabled !== false

  const onSubmitRef = useRef(onSubmit)
  useEffect(() => {
    onSubmitRef.current = onSubmit
  }, [onSubmit])

  const handleSubmit = useCallback(() => {
    form.handleSubmit(onSubmitRef.current)()
  }, [form])

  const handleReset = useCallback(() => {
    form.reset()
  }, [form])

  useEffect(() => {
    if (isDirty && isEnabled) {
      registerForm({
        submit: handleSubmit,
        reset: handleReset,
        isPending,
      })
    } else {
      // If dirty but disabled (e.g. inside a dialog), ensure we unregister
      unregisterForm()
    }

    return () => {
      unregisterForm()
    }
  }, [
    isDirty,
    isPending,
    isEnabled,
    registerForm,
    unregisterForm,
    handleSubmit,
    handleReset,
  ])
}
