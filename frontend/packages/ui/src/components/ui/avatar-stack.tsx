import { cn } from '@/lib/utils'
import { Avatar, AvatarFallback } from './avatar'

const overflowSizes = {
  xs: 'size-6 text-[0.5rem]',
  sm: 'size-8 text-xs',
  default: 'size-10 text-sm',
} as const

export interface AvatarStackProps {
  /** Initials in display order. */
  items: string[]
  /** How many avatars to show before collapsing into a "+N" bubble. */
  max?: number
  size?: keyof typeof overflowSizes
  className?: string
}

const AvatarStack = ({ items, max = 3, size = 'sm', className }: AvatarStackProps) => {
  if (items.length === 0) return null
  const shown = items.slice(0, max)
  const hidden = items.length - shown.length

  return (
    <span className={cn('flex -space-x-2', className)}>
      {shown.map((initials, index) => (
        // Initials repeat across people, so the index has to carry the key.
        <Avatar key={`${initials}-${index}`} size={size} className="ring-2 ring-white">
          <AvatarFallback variant="user">{initials}</AvatarFallback>
        </Avatar>
      ))}
      {hidden > 0 && (
        <span
          className={cn(
            'flex items-center justify-center rounded-full bg-dark-100 font-semibold text-dark-600 ring-2 ring-white',
            overflowSizes[size],
          )}
        >
          +{hidden}
        </span>
      )}
    </span>
  )
}
AvatarStack.displayName = 'AvatarStack'

export { AvatarStack }
