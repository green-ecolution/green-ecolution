import { useId, type ComponentType, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Switch, cn } from '@green-ecolution/ui'
import { CARD } from '../cardStyles'

interface ModuleAreaCardProps {
  icon: ComponentType<{ className?: string }>
  title: string
  description: string | null
  /** Without a switch the card has no on/off state (the sensors card). */
  toggle?: {
    enabled: boolean
    note: string | null
    /** Absent where the organization only follows: the state shows without a switch. */
    control?: {
      disabled: boolean
      onChange: (enabled: boolean) => void
    }
  }
  children?: ReactNode
}

const ModuleAreaCard = ({
  icon: Icon,
  title,
  description,
  toggle,
  children,
}: ModuleAreaCardProps) => {
  const { t } = useTranslation('settings')
  const titleId = useId()
  const noteId = useId()

  return (
    <section aria-labelledby={titleId} className={cn(CARD, 'p-0')}>
      <div className="flex flex-wrap items-center gap-x-4 gap-y-3 px-6 py-4">
        <span
          aria-hidden
          className="flex size-10 flex-none items-center justify-center rounded-lg bg-green-light-200 text-green-dark"
        >
          <Icon className="size-5" />
        </span>
        <div className="min-w-0 flex-[1_1_14rem]">
          <h3 id={titleId} className="font-lato text-lg font-bold text-dark">
            {title}
          </h3>
          {description && <p className="text-sm text-dark-600">{description}</p>}
        </div>
        {toggle && (
          <>
            <span className="text-sm font-semibold text-dark">
              {toggle.enabled ? t('organization.areas.enabled') : t('organization.areas.disabled')}
            </span>
            {toggle.control && (
              <Switch
                aria-labelledby={titleId}
                aria-describedby={toggle.note ? noteId : undefined}
                checked={toggle.enabled}
                disabled={toggle.control.disabled}
                onCheckedChange={toggle.control.onChange}
              />
            )}
          </>
        )}
      </div>
      {toggle?.note && (
        <p id={noteId} className="px-6 pb-4 text-sm text-dark-600">
          {toggle.note}
        </p>
      )}
      {children}
    </section>
  )
}

export default ModuleAreaCard
