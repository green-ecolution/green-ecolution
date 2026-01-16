import * as React from 'react'
import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const listCardVariants = cva(
  'bg-white border border-dark-50 shadow-cards flex transition-all ease-in-out duration-300',
  {
    variants: {
      hoverable: {
        true: 'hover:bg-green-dark-50 hover:border-green-dark',
        false: '',
      },
      size: {
        default: 'p-6 rounded-xl flex-col gap-y-4',
        compact: 'px-4 py-3 rounded-lg items-center gap-x-4',
      },
    },
    defaultVariants: {
      hoverable: true,
      size: 'default',
    },
  }
)

interface ListCardProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof listCardVariants> {
  asChild?: boolean
  columns?: string
}

const ListCard = React.forwardRef<HTMLDivElement, ListCardProps>(
  ({ className, hoverable, size, asChild = false, columns, children, ...props }, ref) => {
    const Comp = asChild ? Slot : 'div'

    const gridStyle = columns
      ? { '--list-card-columns': columns } as React.CSSProperties
      : undefined

    return (
      <Comp
        ref={ref}
        className={cn(
          listCardVariants({ hoverable, size }),
          columns && size !== 'compact' && 'lg:grid lg:items-center lg:gap-5 lg:py-5 xl:px-10 lg:[grid-template-columns:var(--list-card-columns)]',
          className
        )}
        style={gridStyle}
        {...props}
      >
        {children}
      </Comp>
    )
  }
)
ListCard.displayName = 'ListCard'

interface ListCardHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
  columns?: string
}

const ListCardHeader = React.forwardRef<HTMLDivElement, ListCardHeaderProps>(
  ({ className, columns, ...props }, ref) => {
    const gridStyle = columns
      ? { '--list-card-columns': columns } as React.CSSProperties
      : undefined

    return (
      <header
        ref={ref}
        className={cn(
          'hidden border-b pb-2 text-sm text-dark-800 px-6 border-b-dark-200 mb-5',
          columns && 'lg:grid lg:gap-5 xl:px-10 lg:[grid-template-columns:var(--list-card-columns)]',
          className
        )}
        style={gridStyle}
        {...props}
      />
    )
  }
)
ListCardHeader.displayName = 'ListCardHeader'

const ListCardCell = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn('', className)} {...props} />
))
ListCardCell.displayName = 'ListCardCell'

interface ListCardStatusProps extends React.HTMLAttributes<HTMLDivElement> {
  status?: 'green-dark' | 'green-light' | 'yellow' | 'red' | 'dark' | string
}

const ListCardStatus = React.forwardRef<HTMLDivElement, ListCardStatusProps>(
  ({ className, status = 'dark', children, ...props }, ref) => {
    // Normalize status by removing 'outline-' prefix if present
    const normalizedStatus = status.replace(/^outline-/, '')

    const statusColorMap: Record<string, string> = {
      'green-dark': 'bg-green-dark',
      'green-light': 'bg-green-light',
      yellow: 'bg-yellow',
      red: 'bg-red',
      dark: 'bg-dark-400',
    }

    const dotClass = statusColorMap[normalizedStatus] || 'bg-dark-400'

    return (
      <div
        ref={ref}
        className={cn('flex items-center gap-3 font-medium', className)}
        {...props}
      >
        <span className={cn('w-4 h-4 rounded-full shrink-0', dotClass)} />
        {children}
      </div>
    )
  }
)
ListCardStatus.displayName = 'ListCardStatus'

const ListCardTitle = React.forwardRef<
  HTMLHeadingElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <h2 ref={ref} className={cn('font-bold text-lg', className)} {...props} />
))
ListCardTitle.displayName = 'ListCardTitle'

const ListCardDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p ref={ref} className={cn('text-dark-800', className)} {...props} />
))
ListCardDescription.displayName = 'ListCardDescription'

const ListCardMeta = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn('text-dark-800 flex gap-x-2', className)} {...props} />
))
ListCardMeta.displayName = 'ListCardMeta'

const ListCardContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn('flex-1 min-w-0', className)} {...props} />
))
ListCardContent.displayName = 'ListCardContent'

const ListCardActions = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn('flex items-center gap-2 shrink-0', className)} {...props} />
))
ListCardActions.displayName = 'ListCardActions'

export {
  ListCard,
  ListCardHeader,
  ListCardCell,
  ListCardStatus,
  ListCardTitle,
  ListCardDescription,
  ListCardMeta,
  ListCardContent,
  ListCardActions,
  listCardVariants,
}
export type { ListCardProps, ListCardHeaderProps, ListCardStatusProps }
