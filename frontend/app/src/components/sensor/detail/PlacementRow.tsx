import type { ReactNode } from 'react'
import { Link, type LinkProps } from '@tanstack/react-router'
import { ChevronRight } from 'lucide-react'
import { cn } from '@green-ecolution/ui'

interface PlacementRowProps {
  icon: ReactNode
  caption: string
  title: ReactNode
  meta?: ReactNode
  link?: LinkProps
}

/** One line of the placement card; modules use it so their rows look like the core's. */
const PlacementRow = ({ icon, caption, title, meta, link }: PlacementRowProps) => {
  const body = (
    <>
      <span className="grid size-10 shrink-0 place-items-center rounded-lg bg-green-dark-50 text-green-dark [&_svg]:size-5">
        {icon}
      </span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="text-xs text-muted-foreground">{caption}</span>
        <span className="truncate font-semibold">{title}</span>
        {meta && (
          <span className="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-muted-foreground">
            {meta}
          </span>
        )}
      </span>
      {link && (
        <ChevronRight
          className="size-5 shrink-0 text-muted-foreground transition-colors group-hover:text-green-dark"
          aria-hidden
        />
      )}
    </>
  )
  const rowClass = 'flex min-h-16 items-center gap-3 px-3 py-2.5'

  return link ? (
    <Link
      {...link}
      className={cn(
        rowClass,
        'group transition-colors hover:bg-green-dark-50/50 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-green-dark',
      )}
    >
      {body}
    </Link>
  ) : (
    <div className={rowClass}>{body}</div>
  )
}

export default PlacementRow
