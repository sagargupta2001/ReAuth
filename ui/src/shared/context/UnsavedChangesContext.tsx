/* eslint-disable react-refresh/only-export-components */
import { type ReactNode, createContext, useCallback, useContext, useState } from 'react'

interface UnsavedChangesContextType {
  isDirty: boolean
  isPending: boolean
  // Register a form's handlers to the global context
  registerForm: (actions: { submit: () => void; reset: () => void; isPending: boolean }) => void
  // Unregister when form unmounts
  unregisterForm: () => void
  // Actions called by the Floating Bar
  triggerSave: () => void
  triggerReset: () => void
}

const UnsavedChangesContext = createContext<UnsavedChangesContextType | null>(null)

export function UnsavedChangesProvider({ children }: { children: ReactNode }) {
  const [isDirty, setIsDirty] = useState(false)
  const [handlers, setHandlers] = useState<{
    submit: () => void
    reset: () => void
    isPending: boolean
  } | null>(null)

  const registerForm = useCallback(
    (actions: { submit: () => void; reset: () => void; isPending: boolean }) => {
      setIsDirty(true)
      setHandlers(actions)
    },
    [],
  )

  const unregisterForm = useCallback(() => {
    setIsDirty(false)
    setHandlers(null)
  }, [])

  const triggerSave = useCallback(() => {
    handlers?.submit()
  }, [handlers])

  const triggerReset = useCallback(() => {
    handlers?.reset()
    setIsDirty(false) // Optimistically clear dirty state
  }, [handlers])

  return (
    <UnsavedChangesContext.Provider
      value={{
        isDirty,
        isPending: handlers?.isPending || false,
        registerForm,
        unregisterForm,
        triggerSave,
        triggerReset,
      }}
    >
      {children}
    </UnsavedChangesContext.Provider>
  )
}

export function useUnsavedChanges() {
  const context = useContext(UnsavedChangesContext)
  if (!context) {
    throw new Error('useUnsavedChanges must be used within a UnsavedChangesProvider')
  }
  return context
}