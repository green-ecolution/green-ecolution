import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '../../lib/utils'

const KanbanBoard = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        'flex items-stretch gap-4 overflow-x-auto pb-4 snap-x snap-mandatory md:snap-none',
        className,
      )}
      {...props}
    />
  ),
)
KanbanBoard.displayName = 'KanbanBoard'

const kanbanColumnVariants = cva(
  'flex w-[300px] shrink-0 snap-start flex-col gap-3 rounded-xl p-3 transition-[background-color,opacity] duration-200 motion-reduce:transition-none',
  {
    variants: {
      tone: {
        neutral: 'bg-dark-50',
        suggestion: 'bg-green-light-50',
        active: 'bg-green-dark-50',
      },
      state: {
        idle: '',
        target: 'bg-green-dark-100 ring-2 ring-green-dark',
        dimmed: 'opacity-40',
      },
    },
    defaultVariants: { tone: 'neutral', state: 'idle' },
  },
)

export interface KanbanColumnProps
  extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof kanbanColumnVariants> {}

const KanbanColumn = React.forwardRef<HTMLDivElement, KanbanColumnProps>(
  ({ className, tone, state, ...props }, ref) => (
    <section
      ref={ref}
      className={cn(kanbanColumnVariants({ tone, state }), className)}
      {...props}
    />
  ),
)
KanbanColumn.displayName = 'KanbanColumn'

interface KanbanColumnHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
  icon?: React.ReactNode
  title: string
  count?: number
}

const KanbanColumnHeader = ({
  icon,
  title,
  count,
  className,
  ...props
}: KanbanColumnHeaderProps) => (
  <div
    className={cn('flex items-center gap-2 px-1 pt-1 text-dark-800 [&>svg]:size-4', className)}
    {...props}
  >
    {icon}
    <h2 className="font-lato text-sm font-semibold">{title}</h2>
    {count !== undefined && (
      <span className="ml-auto rounded-full bg-white px-2 py-0.5 text-xs font-semibold tabular-nums text-dark-600">
        {count}
      </span>
    )}
  </div>
)

const kanbanCardVariants = cva(
  'rounded-xl border border-dark-100 bg-white p-4 shadow-cards transition-[box-shadow,transform] duration-200 motion-reduce:transition-none motion-reduce:transform-none',
  {
    variants: {
      state: {
        idle: '',
        dragging: 'rotate-2 shadow-lg',
        ghost: 'opacity-40',
      },
    },
    defaultVariants: { state: 'idle' },
  },
)

export interface KanbanCardProps
  extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof kanbanCardVariants> {}

const KanbanCard = React.forwardRef<HTMLDivElement, KanbanCardProps>(
  ({ className, state, ...props }, ref) => (
    <div ref={ref} className={cn(kanbanCardVariants({ state }), className)} {...props} />
  ),
)
KanbanCard.displayName = 'KanbanCard'

const KanbanDropHint = ({
  label,
  className,
  ...props
}: { label: string } & React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      'flex h-16 items-center justify-center rounded-xl border-2 border-dashed border-green-dark bg-white/60 text-sm font-semibold text-green-dark',
      className,
    )}
    {...props}
  >
    {label}
  </div>
)

const KanbanColumnEmpty = ({ className, ...props }: React.HTMLAttributes<HTMLParagraphElement>) => (
  <p
    className={cn(
      'flex flex-1 flex-col items-center justify-center rounded-xl border border-dashed border-dark-200 p-4 text-center text-sm text-dark-600',
      className,
    )}
    {...props}
  />
)

export {
  KanbanBoard,
  KanbanColumn,
  KanbanColumnHeader,
  KanbanCard,
  KanbanDropHint,
  KanbanColumnEmpty,
  kanbanColumnVariants,
  kanbanCardVariants,
}
