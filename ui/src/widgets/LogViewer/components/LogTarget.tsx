import { ChevronRight } from 'lucide-react'

interface Props {
  target: string
}

/**
 * A small component to render a single part of the log target path.
 */
function TargetPart({ name, isLast = false }: { name: string; isLast?: boolean }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className={isLast ? 'text-foreground font-medium' : 'text-muted-foreground'}>
        {name}
      </span>
      {!isLast && <ChevronRight className="text-muted-foreground h-3.5 w-3.5" />}
    </div>
  )
}

/**
 * Renders the target string directly as readable path segments.
 */
export function LogTarget({ target }: Props) {
  const pathParts = target.split('::').filter((p) => p.length > 0)

  return (
    <div className="flex items-center gap-1.5 font-mono text-xs">
      {pathParts.map((part, i) => (
        <TargetPart key={i} name={part} isLast={i === pathParts.length - 1} />
      ))}
    </div>
  )
}
