import { useCallback, useMemo } from 'react'

import { useSearchParams } from 'react-router-dom'

export type ParamCodec<T> = {
  parse: (value: string | null) => T
  serialize: (value: T) => string | undefined
}

export function stringParam(defaultValue = ''): ParamCodec<string> {
  return {
    parse: (value) => value ?? defaultValue,
    serialize: (value) => {
      if (!value || value === defaultValue) return undefined
      return value
    },
  }
}

export function numberParam(defaultValue: number): ParamCodec<number> {
  return {
    parse: (value) => {
      if (!value) return defaultValue
      const parsed = Number(value)
      return Number.isNaN(parsed) ? defaultValue : parsed
    },
    serialize: (value) => (value === defaultValue ? undefined : String(value)),
  }
}

export function enumParam<T extends string>(values: readonly T[], defaultValue: T): ParamCodec<T> {
  return {
    parse: (value) => {
      if (!value) return defaultValue
      return (values.includes(value as T) ? value : defaultValue) as T
    },
    serialize: (value) => (value === defaultValue ? undefined : value),
  }
}

export function booleanParam(defaultValue = false): ParamCodec<boolean> {
  return {
    parse: (value) => {
      if (!value) return defaultValue
      return value === 'true'
    },
    serialize: (value) => (value === defaultValue ? undefined : String(value)),
  }
}

export function useUrlState<T extends Record<string, unknown>>(
  config: { [K in keyof T]: ParamCodec<T[K]> },
) {
  const [searchParams, setSearchParams] = useSearchParams()

  const state = useMemo(() => {
    const next: Record<string, unknown> = {}
    Object.entries(config).forEach(([key, codec]) => {
      next[key] = (codec as ParamCodec<unknown>).parse(searchParams.get(key))
    })
    return next as T
  }, [config, searchParams])

  const setState = useCallback(
    (updates: Partial<T>) => {
      const params = new URLSearchParams(searchParams)

      Object.entries(updates).forEach(([key, value]) => {
        const codec = config[key as keyof T]
        if (!codec) return
        const serialized = codec.serialize(value as T[keyof T])
        if (serialized === undefined) {
          params.delete(key)
        } else {
          params.set(key, serialized)
        }
      })

      setSearchParams(params)
    },
    [config, searchParams, setSearchParams],
  )

  return [state, setState] as const
}
