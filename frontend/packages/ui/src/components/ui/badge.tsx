import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const badgeVariants = cva(
  'inline-flex items-center rounded-full border transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-primary text-primary-foreground',
        secondary: 'border-transparent bg-secondary text-secondary-foreground',
        destructive: 'border-transparent bg-destructive text-destructive-foreground',
        outline: 'text-foreground',
        success: 'border-green-dark bg-green-dark-50 text-green-dark',
        warning: 'border-yellow bg-yellow-50 text-yellow-900',
        error: 'border-red bg-red-50 text-red',
        muted: 'border-dark-400 bg-dark-50 text-dark-600',
        'green-dark': 'border-green-dark bg-green-dark-50 text-green-dark',
        'green-light': 'border-green-light bg-green-light-50 text-green-light-900',
        // Outline variants (formerly Pill)
        'outline-red': 'border-red text-red',
        'outline-yellow': 'border-yellow text-yellow',
        'outline-dark': 'border-dark-600 text-dark-600',
        'outline-green-dark': 'border-green-dark text-green-dark',
        'outline-green-light': 'border-green-light bg-green-light-200 text-green-dark',
      },
      size: {
        default: 'px-2.5 py-0.5 text-xs font-semibold',
        lg: 'px-4 py-1 text-sm font-medium',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, size, ...props }: BadgeProps) {
  return <div className={cn(badgeVariants({ variant, size }), className)} {...props} />
}

export { Badge, badgeVariants }
