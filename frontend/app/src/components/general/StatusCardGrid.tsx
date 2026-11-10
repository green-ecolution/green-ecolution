import type { ReactNode } from 'react'
import { cn } from '@green-ecolution/ui'

/** Renders a `<ul>`; children must be `<li>` elements. */
interface StatusCardGridProps {
  columns?: 2 | 3 | 4
  className?: string
  children: ReactNode
}

const StatusCardGrid = ({ columns = 3, className, children }: StatusCardGridProps) => (
  <ul
    className={cn(
      'flex flex-col gap-y-5 md:grid md:gap-5 md:grid-cols-2',
      { 2: 'lg:grid-cols-2', 3: 'lg:grid-cols-3', 4: 'lg:grid-cols-4' }[columns],
      className,
    )}
  >
    {children}
  </ul>
)

export default StatusCardGrid
