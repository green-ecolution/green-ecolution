import { useTranslation } from 'react-i18next'
import { ArrowLeftRight, Plus, Shield, UserRound } from 'lucide-react'
import { Button, EmptyState } from '@green-ecolution/ui'
import { initialsOfName } from '@/lib/initials'

interface MembersEmptyStateProps {
  organizationName: string
  variant: 'onlyMe' | 'none'
  viewerName: string | null
  onAdd?: () => void
  onEditOwnProfile?: () => void
}

const ExampleRow = ({ dashed, children }: { dashed?: boolean; children: React.ReactNode }) => (
  <div
    className={`flex min-h-[52px] items-center gap-3 rounded-[10px] border px-3.5 ${
      dashed ? 'border-dashed border-dark-400 bg-dark-50/50' : 'border-dark-200 bg-white'
    }`}
  >
    {children}
  </div>
)

const NewPersonRow = ({ name, role }: { name: string; role: string }) => (
  <ExampleRow dashed>
    <span className="flex size-8 shrink-0 items-center justify-center rounded-full bg-green-light-100 text-green-dark">
      <UserRound className="size-4" strokeWidth={1.5} />
    </span>
    <span className="min-w-0 flex-1 truncate font-bold text-dark-600">{name}</span>
    <span className="rounded-full border border-dark-400 bg-white px-2.5 text-sm text-dark-600">
      {role}
    </span>
  </ExampleRow>
)

const MembersEmptyState = ({
  organizationName,
  variant,
  viewerName,
  onAdd,
  onEditOwnProfile,
}: MembersEmptyStateProps) => {
  const { t } = useTranslation('settings')
  const prefix = 'members.empty'
  const onlyMe = variant === 'onlyMe'

  return (
    <EmptyState
      title={t(onlyMe ? `${prefix}.titleOnlyMe` : `${prefix}.titleNone`, {
        name: organizationName,
      })}
      description={t(`${prefix}.description`, { name: organizationName })}
      action={
        (onAdd ?? onEditOwnProfile) && (
          <div className="flex flex-wrap gap-3">
            {onAdd && (
              <Button type="button" onClick={onAdd}>
                <Plus className="size-4" aria-hidden />
                {t('members.add.button')}
              </Button>
            )}
            {onEditOwnProfile && (
              <Button type="button" variant="outline" onClick={onEditOwnProfile}>
                {t(`${prefix}.ownProfile`)}
              </Button>
            )}
          </div>
        )
      }
      example={{
        caption: t(`${prefix}.exampleCaption`),
        label: t(onlyMe ? `${prefix}.exampleLabel` : `${prefix}.exampleLabelNone`),
        content: (
          <div className="flex flex-col gap-2">
            {onlyMe && viewerName && (
              <ExampleRow>
                <span className="flex size-8 shrink-0 items-center justify-center rounded-full bg-green-dark text-xs font-bold text-white">
                  {initialsOfName(viewerName)}
                </span>
                <span className="min-w-0 flex-1 truncate font-bold">{viewerName}</span>
                <span className="rounded-full border border-green-dark px-2.5 text-sm font-bold text-green-dark">
                  {t(`${prefix}.exampleYou`)}
                </span>
              </ExampleRow>
            )}
            <NewPersonRow
              name={t(`${prefix}.examplePerson`)}
              role={t(`${prefix}.exampleRoleFirst`)}
            />
            <NewPersonRow
              name={t(`${prefix}.examplePerson`)}
              role={t(`${prefix}.exampleRoleSecond`)}
            />
          </div>
        ),
      }}
      benefits={{
        title: t(`${prefix}.benefitsTitle`),
        items: [
          {
            icon: <UserRound className="size-[22px]" strokeWidth={1.5} />,
            title: t(`${prefix}.accessTitle`),
            text: t(`${prefix}.accessText`),
          },
          {
            icon: <Shield className="size-[22px]" strokeWidth={1.5} />,
            title: t(`${prefix}.rolesTitle`),
            text: t(`${prefix}.rolesText`),
          },
          {
            icon: <ArrowLeftRight className="size-[22px]" strokeWidth={1.5} />,
            title: t(`${prefix}.plansTitle`),
            text: t(`${prefix}.plansText`),
          },
        ],
      }}
      note={t(`${prefix}.note`)}
    />
  )
}

export default MembersEmptyState
