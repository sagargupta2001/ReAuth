import { describe, expect, it } from 'vitest'

import {
  actionKey,
  appendAction,
  patchAction,
  patchActionSignal,
  payloadEntriesOf,
  rememberActionNodeId,
  removeActionById,
  setPayloadMap,
  validatePayloadPath,
  withActionIds,
  type InspectorAction,
} from './actionBindings'

const INPUTS = ['email', 'password']

describe('validatePayloadPath', () => {
  it('treats an empty path as valid, since the field is optional', () => {
    expect(validatePayloadPath('', INPUTS).valid).toBe(true)
    expect(validatePayloadPath('   ', INPUTS).valid).toBe(true)
  })

  it('accepts a known input', () => {
    expect(validatePayloadPath('inputs.email', INPUTS).valid).toBe(true)
  })

  it('rejects an unknown input by name', () => {
    const result = validatePayloadPath('inputs.nope', INPUTS)
    expect(result.valid).toBe(false)
    expect(result.message).toContain("'nope'")
  })

  it('rejects empty segments', () => {
    expect(validatePayloadPath('inputs.email.', INPUTS).valid).toBe(false)
    expect(validatePayloadPath('context.a..b', INPUTS).valid).toBe(false)
  })

  it('requires something after context.', () => {
    expect(validatePayloadPath('context.', INPUTS).valid).toBe(false)
    expect(validatePayloadPath('context.client_id', INPUTS).valid).toBe(true)
  })

  it('rejects a path with no recognised root', () => {
    const result = validatePayloadPath('email', INPUTS)
    expect(result.valid).toBe(false)
    expect(result.message).toMatch(/inputs\.|context\./)
  })
})

describe('withActionIds / actionKey', () => {
  it('fills in a missing id', () => {
    const [action] = withActionIds([{ trigger: 'on_click' }])
    expect(action.action_id).toBeTruthy()
  })

  it('keeps an existing id', () => {
    const [action] = withActionIds([{ action_id: 'keep-me' }])
    expect(action.action_id).toBe('keep-me')
  })

  it('falls back to the index for an id-less action', () => {
    expect(actionKey({}, 2)).toBe('action-2')
    expect(actionKey({ action_id: 'a1' }, 2)).toBe('a1')
  })
})

describe('patchAction / patchActionSignal', () => {
  const actions: InspectorAction[] = [
    { action_id: 'a1', trigger: 'on_click', signal: { type: 'submit_node' } },
    { action_id: 'a2', trigger: 'on_load' },
  ]

  it('patches only the targeted action', () => {
    const next = patchAction(actions, 'a1', { trigger: 'on_submit' })
    expect(next[0].trigger).toBe('on_submit')
    expect(next[1].trigger).toBe('on_load')
  })

  it('merges into the signal rather than replacing it', () => {
    const next = patchActionSignal(actions, 'a1', { node_id: 'auth-password' })
    expect(next[0].signal).toEqual({ type: 'submit_node', node_id: 'auth-password' })
  })

  it('creates the signal when absent', () => {
    const next = patchActionSignal(actions, 'a2', { type: 'validate_node' })
    expect(next[1].signal).toEqual({ type: 'validate_node' })
  })

  it('does not mutate the input', () => {
    patchAction(actions, 'a1', { trigger: 'on_submit' })
    expect(actions[0].trigger).toBe('on_click')
  })
})

describe('payload map round-trip', () => {
  const actions: InspectorAction[] = [
    { action_id: 'a1', signal: { type: 'submit_node', payload_map: { email: 'inputs.email' } } },
  ]

  it('reads entries out', () => {
    expect(payloadEntriesOf(actions[0])).toEqual([{ key: 'email', path: 'inputs.email' }])
  })

  it('returns nothing when there is no map', () => {
    expect(payloadEntriesOf({ action_id: 'x' })).toEqual([])
  })

  it('writes entries back, trimming keys and paths', () => {
    const next = setPayloadMap(actions, 'a1', [{ key: '  user ', path: ' inputs.email ' }])
    expect(next[0].signal?.payload_map).toEqual({ user: 'inputs.email' })
  })

  it('drops entries with a blank key', () => {
    const next = setPayloadMap(actions, 'a1', [
      { key: 'kept', path: 'inputs.email' },
      { key: '   ', path: 'inputs.password' },
    ])
    expect(next[0].signal?.payload_map).toEqual({ kept: 'inputs.email' })
  })

  it('removes payload_map entirely when it empties, rather than persisting {}', () => {
    const next = setPayloadMap(actions, 'a1', [])
    expect(next[0].signal).toEqual({ type: 'submit_node' })
    expect(next[0].signal).not.toHaveProperty('payload_map')
  })
})

describe('appendAction / removeActionById', () => {
  it('appends an action with an id and sensible defaults', () => {
    const next = appendAction([])
    expect(next).toHaveLength(1)
    expect(next[0].action_id).toBeTruthy()
    expect(next[0].trigger).toBe('on_click')
    expect(next[0].signal?.type).toBe('submit_node')
  })

  it('removes by id and by index fallback', () => {
    expect(removeActionById([{ action_id: 'a1' }, { action_id: 'a2' }], 'a1')).toEqual([
      { action_id: 'a2' },
    ])
    expect(removeActionById([{}, { action_id: 'a2' }], 'action-0')).toEqual([
      { action_id: 'a2' },
    ])
  })
})

describe('rememberActionNodeId', () => {
  it('puts the newest first and de-duplicates', () => {
    expect(rememberActionNodeId('b', ['a', 'b', 'c'])).toEqual(['b', 'a', 'c'])
  })

  it('ignores blank input', () => {
    expect(rememberActionNodeId('  ', ['a'])).toEqual(['a'])
  })

  it('caps the list', () => {
    const many = Array.from({ length: 25 }, (_, i) => `id-${i}`)
    expect(rememberActionNodeId('new', many)).toHaveLength(20)
  })
})
