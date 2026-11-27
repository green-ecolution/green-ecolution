import { useTranslation } from 'react-i18next'
import { Eye, Lock, Plus, Users } from 'lucide-react'
import { Button, EmptyState } from '@green-ecolution/ui'
import { initialsOfName } from '@/lib/initials'

interface StructureEmptyStateProps {
  organizationName: string
  onCreate?: () => void
}

const Branch = () => (
  <span
    aria-hidden
    className="-mt-3 h-3.5 w-3.5 shrink-0 rounded-bl-md border-b-2 border-l-2 border-dark-300"
  />
)

const SubOrganizationTile = ({ name }: { name: string }) => (
  <div className="flex items-center gap-2.5 pl-5">
    <Branch />
    <div className="flex min-h-[52px] flex-1 items-center gap-3 rounded-[10px] border border-dashed border-dark-400 bg-dark-50/50 px-3.5">
      <span className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-green-light-100 text-sm font-bold text-green-dark">
        {initialsOfName(name)}
      </span>
      <span className="min-w-0 truncate font-lato font-bold text-dark-800">{name}</span>
    </div>
  </div>
)

const StructureEmptyState = ({ organizationName, onCreate }: StructureEmptyStateProps) => {
  const { t } = useTranslation('settings')
  const prefix = 'organization.structure.empty'

  return (
    <EmptyState
      title={t(`${prefix}.title`, { name: organizationName })}
      description={t(`${prefix}.description`, { name: organizationName })}
      action={
        onCreate && (
          <Button type="button" onClick={onCreate}>
            <Plus className="size-4" aria-hidden />
            {t('organization.createSubOrganization')}
          </Button>
        )
      }
      example={{
        caption: t(`${prefix}.exampleCaption`),
        label: t(`${prefix}.exampleLabel`, { name: organizationName }),
        content: (
          <div className="flex flex-col gap-2">
            <div className="flex min-h-[52px] items-center gap-3 rounded-[10px] border border-dark-200 bg-white px-3.5">
              <span className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-green-dark text-sm font-bold text-white">
                {initialsOfName(organizationName)}
              </span>
              <span className="min-w-0 truncate font-lato font-bold">{organizationName}</span>
            </div>
            <SubOrganizationTile name={t(`${prefix}.exampleFirst`)} />
            <SubOrganizationTile name={t(`${prefix}.exampleSecond`)} />
          </div>
        ),
      }}
      benefits={{
        title: t(`${prefix}.benefitsTitle`),
        items: [
          {
            icon: <Users className="size-[22px]" strokeWidth={1.5} />,
            title: t(`${prefix}.teamTitle`),
            text: t(`${prefix}.teamText`),
          },
          {
            icon: <Lock className="size-[22px]" strokeWidth={1.5} />,
            title: t(`${prefix}.dataTitle`),
            text: t(`${prefix}.dataText`),
          },
          {
            icon: <Eye className="size-[22px]" strokeWidth={1.5} />,
            title: t(`${prefix}.overviewTitle`),
            text: t(`${prefix}.overviewText`, { name: organizationName }),
          },
        ],
      }}
      note={t(`${prefix}.note`)}
    />
  )
}

export default StructureEmptyState
