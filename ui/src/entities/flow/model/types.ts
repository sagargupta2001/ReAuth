export type FlowType = 'browser' | 'registration' | 'direct' | 'reset'

export interface Flow {
  id: string
  alias: string
  description: string
  type: FlowType // e.g. "browser"
  builtIn: boolean // true = cannot be deleted, only cloned
  // In a real app, you'd fetch the realm config to know if this is the active one
  isDefault?: boolean
}
