import { useCallback, useMemo } from 'react'

import { useDndContext, useDraggable, useDroppable } from '@dnd-kit/core'
import { ChevronRight, Folder, GripVertical, MoreVertical, Plus } from 'lucide-react'

import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { cn } from '@/lib/utils'
import type { FlattenedGroupNode } from '@/features/group-tree/model/types'
import { indentationWidth } from '@/features/group-tree/lib/tree-utils'

interface GroupTreeItemProps {
  item: FlattenedGroupNode
  isExpanded: boolean
  isSelected: boolean
  onToggle: (id: string) => void
  onSelect: (id: string) => void
  onCreateChild: (parentId: string) => void
  onMoveToRoot?: (id: string) => void
  onDelete?: (id: string) => void
}

export function GroupTreeItem({
  item,
  isExpanded,
  isSelected,
  onToggle,
  onSelect,
  onCreateChild,
  onMoveToRoot,
  onDelete,
}: GroupTreeItemProps) {
  const { active } = useDndContext()
  const {
    attributes,
    listeners,
    setNodeRef: setDraggableNodeRef,
    setActivatorNodeRef,
    isDragging,
  } = useDraggable({ id: item.id })
  const { isOver, setNodeRef: setDroppableNodeRef } = useDroppable({ id: item.id })
  const setNodeRef = useCallback(
    (node: HTMLDivElement | null) => {
      setDraggableNodeRef(node)
      setDroppableNodeRef(node)
    },
    [setDraggableNodeRef, setDroppableNodeRef],
  )
  const isHovered = isOver && active?.id !== item.id

  const paddingLeft = useMemo(() => item.depth * indentationWidth, [item.depth])

  return (
    <div
      ref={setNodeRef}
      className={cn(
        'group flex items-center gap-2 rounded-md px-2 py-1 text-sm transition-colors',
        isSelected && 'bg-primary/10 text-primary',
        isHovered &&
          !isSelected &&
          'bg-primary/15 ring-2 ring-primary/60 ring-offset-2 ring-offset-background',
        isDragging && 'opacity-60',
      )}
    >
      <div className="flex items-center" style={{ paddingLeft }}>
        {item.has_children ? (
          <button
            type="button"
            onClick={() => onToggle(item.id)}
            className="text-muted-foreground hover:text-foreground mr-1 flex h-5 w-5 items-center justify-center"
          >
            <ChevronRight
              className={cn('h-4 w-4 transition-transform', isExpanded && 'rotate-90')}
            />
          </button>
        ) : (
          <span className="mr-1 h-5 w-5" />
        )}
      </div>

      <button
        type="button"
        onClick={() => onSelect(item.id)}
        className="flex flex-1 items-center gap-2 text-left"
      >
        <Folder className="text-muted-foreground h-4 w-4" />
        <span className="truncate">{item.name}</span>
      </button>

      <div className="flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => onCreateChild(item.id)}
        >
          <Plus className="h-4 w-4" />
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button type="button" variant="ghost" size="icon" className="h-7 w-7">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => onCreateChild(item.id)}>
              Add sub-group
            </DropdownMenuItem>
            <DropdownMenuItem
              disabled={!onMoveToRoot || item.parentId === null}
              onClick={() => onMoveToRoot?.(item.id)}
            >
              Move to root
            </DropdownMenuItem>
            <DropdownMenuItem disabled={!onDelete} onClick={() => onDelete?.(item.id)}>
              Delete
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <button
          type="button"
          className="text-muted-foreground hover:text-foreground flex h-7 w-7 items-center justify-center"
          ref={setActivatorNodeRef}
          {...attributes}
          {...listeners}
        >
          <GripVertical className="h-4 w-4" />
        </button>
      </div>
    </div>
  )
}
