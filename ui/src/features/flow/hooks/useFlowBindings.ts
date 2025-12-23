import { useMemo } from 'react'

import type { UnifiedFlowDto } from '@/entities/flow/model/types'
import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'

// Updated keys to match your API response (suffix: _id)
const BINDING_KEYS = [
  // Standard Keys (in case API changes)
  'browserFlow',
  'browser_flow',
  'registrationFlow',
  'registration_flow',
  'directGrantFlow',
  'direct_grant_flow',
  'resetCredentialsFlow',
  'reset_credentials_flow',

  // YOUR API Specific Keys (Suffix _id)
  'browser_flow_id',
  'registration_flow_id',
  'direct_grant_flow_id',
  'reset_credentials_flow_id',
  'client_authentication_flow_id',
  'docker_authentication_flow_id',
] as const

export function useFlowBindings() {
  const realmId = useActiveRealm()

  // Use the list hook since we know it contains the data we need
  const { data: realmData } = useCurrentRealm()

  const boundValues = useMemo(() => {
    if (!realmData) return new Set<string>()

    const activeSet = new Set<string>()

    BINDING_KEYS.forEach((key) => {
      // @ts-ignore - Dynamic roles
      const value = realmData[key]

      // We accept both IDs (UUIDs) and Aliases (strings)
      if (value && typeof value === 'string') {
        activeSet.add(value)
      }
    })

    return activeSet
  }, [realmData])

  /**
   * Checks if a flow is active by comparing its ID or Alias
   * against the values found in the realm config.
   */
  const isFlowActive = (flow: UnifiedFlowDto) => {
    if (boundValues.size === 0) return false

    // 1. Check ID (Most likely match since your API returns UUIDs)
    if (boundValues.has(flow.id)) return true

    // 2. Check Alias (Exact)
    if (boundValues.has(flow.alias)) return true

    // 3. Check Alias/ID (Case-insensitive fallback)
    for (const boundVal of boundValues) {
      if (boundVal.toLowerCase() === flow.alias.toLowerCase()) return true
      if (boundVal.toLowerCase() === flow.id.toLowerCase()) return true
    }

    return false
  }

  return {
    isFlowActive,
    realmId,
    realmData,
  }
}
