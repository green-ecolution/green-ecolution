import { Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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
}: OrganizationTreeProps) => {
  const { t } = useTranslation('settings')

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-4">
        <h2 className="font-lato text-base font-semibold">{t('organization.treeTitle')}</h2>
        {canCreate && (
          <Button type="button" size="sm" onClick={onCreate}>
            <Plus className="size-4" aria-hidden />
            {t('organization.createButton')}
          </Button>
        )}
      </div>

      <ul aria-label={t('organization.treeAriaLabel')} className="flex list-none flex-col gap-1">
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
}

export default OrganizationTree
