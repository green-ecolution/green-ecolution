import { Plus } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import type { OrganizationResponse } from '@/api/backendApi'
import type { OrgNode } from './organizationTree'
import OrganizationTreeItem from './OrganizationTreeItem'

interface OrganizationTreeProps {
  root: OrgNode
  selectedId: string | null
  expanded: ReadonlySet<string>
  canCreate: boolean
  onSelect: (org: OrganizationResponse) => void
  onToggle: (orgId: string) => void
  onCreate: () => void
}

const OrganizationTree = ({
  root,
  selectedId,
  expanded,
  canCreate,
  onSelect,
  onToggle,
  onCreate,
}: OrganizationTreeProps) => (
  <div className="flex flex-col gap-4">
    <div className="flex items-center justify-between gap-4">
      <h2 className="font-lato text-base font-semibold">Organisationsstruktur</h2>
      {canCreate && (
        <Button type="button" size="sm" onClick={onCreate}>
          <Plus className="size-4" aria-hidden />
          Neu
        </Button>
      )}
    </div>

    <ul aria-label="Organisationsstruktur" className="flex list-none flex-col gap-1">
      <OrganizationTreeItem
        node={root}
        depth={0}
        selectedId={selectedId}
        expanded={expanded}
        onSelect={onSelect}
        onToggle={onToggle}
      />
    </ul>
  </div>
)

export default OrganizationTree
