import type { ReactNode } from 'react'
import type { LinkProps } from '@tanstack/react-router'
import { cn } from '@green-ecolution/ui'
import { Pencil } from 'lucide-react'
import BackLink from './links/BackLink'
import ButtonLink from './links/ButtonLink'

interface EntityDetailHeaderProps {
  backLink: { link: LinkProps; label: string }
  title: ReactNode
  badge?: ReactNode
  editLink?: { link: LinkProps; label: string }
  /** Rendered below the edit button, e.g. a destructive delete action. */
  actions?: ReactNode
  /**
   * Controls the row alignment on large screens: '2xl' vertically centers
   * title and actions from 2xl upwards, 'xl' keeps them top-aligned. The
   * difference is inherited from the original call sites and must be kept
   * for visual parity.
   */
  breakpoint?: '2xl' | 'xl'
  children?: ReactNode
}

const EntityDetailHeader = ({
  backLink,
  title,
  badge,
  editLink,
  actions,
  breakpoint = '2xl',
  children,
}: EntityDetailHeaderProps) => (
  <>
    <BackLink link={backLink.link} label={backLink.label} />
    <article
      className={cn(
        'flex flex-col gap-y-6 md:flex-row md:items-start md:justify-between md:gap-x-10',
        breakpoint === '2xl' && '2xl:items-center',
      )}
    >
      <div className={breakpoint === 'xl' ? 'xl:w-4/5' : '2xl:w-4/5'}>
        <h1
          className={cn(
            'font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl',
            badge && 'flex flex-wrap items-center gap-4',
          )}
        >
          {title}
          {badge}
        </h1>
        {children}
      </div>
      {(editLink ?? actions) && (
        <div className="flex shrink-0 flex-col gap-2">
          {editLink && (
            <ButtonLink
              icon={Pencil}
              iconClassName="stroke-1"
              label={editLink.label}
              color="grey"
              link={editLink.link}
            />
          )}
          {actions}
        </div>
      )}
    </article>
  </>
)

export default EntityDetailHeader
