import {
  type Dispatch,
  type ReactNode,
  type SetStateAction,
  createContext,
  useContext,
  useEffect,
  useState,
} from 'react'

import { OmniCommandPalette } from '@/features/Search/components/OmniCommandPalette'

type SearchContextType = {
  open: boolean
  setOpen: Dispatch<SetStateAction<boolean>>
}

const SearchContext = createContext<SearchContextType | null>(null)

type SearchProviderProps = {
  children: ReactNode
}

export function SearchProvider({ children }: SearchProviderProps) {
  const [open, setOpen] = useState(false)

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        setOpen((open) => !open)
      }
    }
    document.addEventListener('keydown', down)
    return () => document.removeEventListener('keydown', down)
  }, [])

  return (
    <SearchContext value={{ open, setOpen }}>
      {children}
      <OmniCommandPalette />
    </SearchContext>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
export const useSearch = () => {
  const searchContext = useContext(SearchContext)

  if (!searchContext) {
    throw new Error('useSearch has to be used within SearchProvider')
  }

  return searchContext
}
