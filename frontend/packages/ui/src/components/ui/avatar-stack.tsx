import type { VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'
import { Avatar, AvatarFallback, avatarVariants } from './avatar'

export interface AvatarStackProps {
  /** Initials in display order. */
  items: string[]
  /** How many avatars to show before collapsing into a "+N" bubble. */
  max?: number
  size?: VariantProps<typeof avatarVariants>['size']
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
            avatarVariants({ size }),
            'items-center justify-center bg-dark-100 font-semibold text-dark-600 ring-2 ring-white',
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
