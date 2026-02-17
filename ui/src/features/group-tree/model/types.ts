export interface GroupTreeNode {
  id: string
  parent_id: string | null
  name: string
  description?: string | null
  sort_order: number
  has_children: boolean
  children?: GroupTreeNode[]
}

export interface FlattenedGroupNode extends GroupTreeNode {
  depth: number
  parentId: string | null
  index: number
}
