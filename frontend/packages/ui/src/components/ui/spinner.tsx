import * as React from 'react'
import { Loader2 } from 'lucide-react'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const spinnerVariants = cva('animate-spin text-muted-foreground', {
  variants: {
    size: {
      default: 'h-6 w-6',
      sm: 'h-4 w-4',
      lg: 'h-8 w-8',
      xl: 'h-12 w-12',
    },
  },
  defaultVariants: {
    size: 'default',
  },
})

export interface SpinnerProps
  extends React.HTMLAttributes<SVGSVGElement>,
    VariantProps<typeof spinnerVariants> {}

const Spinner = React.forwardRef<SVGSVGElement, SpinnerProps>(
  ({ className, size, ...props }, ref) => {
    return (
      <Loader2
        ref={ref}
        className={cn(spinnerVariants({ size, className }))}
        {...props}
      />
    )
  }
)
Spinner.displayName = 'Spinner'

export interface LoadingProps extends React.HTMLAttributes<HTMLDivElement> {
  label?: string
  size?: VariantProps<typeof spinnerVariants>['size']
}

const Loading = React.forwardRef<HTMLDivElement, LoadingProps>(
  ({ className, label, size, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn('flex items-center gap-2', className)}
        {...props}
      >
        <Spinner size={size} />
        {label && <span className="text-sm text-muted-foreground">{label}</span>}
      </div>
    )
  }
)
Loading.displayName = 'Loading'

export { Spinner, Loading, spinnerVariants }
