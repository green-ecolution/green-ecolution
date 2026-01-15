import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { AlertCircle, CheckCircle2, Info, TriangleAlert } from 'lucide-react'

import { cn } from '@/lib/utils'

const alertVariants = cva(
  [
    'relative rounded-xl border shadow-cards',
    'transition-all duration-200 ease-out',
    'animate-in fade-in-0 slide-in-from-top-1',
  ],
  {
    variants: {
      variant: {
        default: 'border-dark-100 bg-white text-foreground',
        info: 'border-green-light-200 bg-green-light-50 border-l-4 border-l-green-light',
        destructive: 'border-red-100 bg-red-50 border-l-4 border-l-red',
        warning: 'border-yellow-100 bg-yellow-50 border-l-4 border-l-yellow',
        success: 'border-green-dark-100 bg-green-dark-50 border-l-4 border-l-green-dark',
      },
      size: {
        default: 'w-full p-6',
        compact: 'w-fit px-4 py-3 rounded-2xl border-l-0 shadow-none',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

export interface AlertProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof alertVariants> {
  /** For dynamic alerts, set to "polite" or "assertive" for screen reader announcements */
  'aria-live'?: 'polite' | 'assertive' | 'off'
}

const Alert = React.forwardRef<HTMLDivElement, AlertProps>(
  ({ className, variant, size, 'aria-live': ariaLive, ...props }, ref) => (
    <div
      ref={ref}
      role="alert"
      aria-live={ariaLive}
      className={cn(alertVariants({ variant, size }), className)}
      {...props}
    />
  ),
)
Alert.displayName = 'Alert'

const alertIconVariants = cva('size-5 shrink-0', {
  variants: {
    variant: {
      default: 'text-muted-foreground',
      info: 'text-green-light',
      destructive: 'text-red',
      warning: 'text-yellow',
      success: 'text-green-dark',
    },
  },
  defaultVariants: {
    variant: 'default',
  },
})

const variantIcons = {
  default: Info,
  info: Info,
  destructive: AlertCircle,
  warning: TriangleAlert,
  success: CheckCircle2,
} as const

export interface AlertIconProps
  extends React.HTMLAttributes<SVGElement>,
    VariantProps<typeof alertIconVariants> {}

const AlertIcon = React.forwardRef<SVGSVGElement, AlertIconProps>(
  ({ className, variant = 'default', ...props }, ref) => {
    const IconComponent = variantIcons[variant ?? 'default']
    return (
      <IconComponent
        ref={ref}
        className={cn(alertIconVariants({ variant }), className)}
        aria-hidden="true"
        {...props}
      />
    )
  },
)
AlertIcon.displayName = 'AlertIcon'

const AlertContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn('flex-1 space-y-1', className)} {...props} />
  ),
)
AlertContent.displayName = 'AlertContent'

const AlertTitle = React.forwardRef<HTMLParagraphElement, React.HTMLAttributes<HTMLHeadingElement>>(
  ({ className, ...props }, ref) => (
    <h5
      ref={ref}
      className={cn(
        'font-lato text-base font-semibold leading-tight tracking-tight text-foreground',
        className,
      )}
      {...props}
    />
  ),
)
AlertTitle.displayName = 'AlertTitle'

const AlertDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p
    ref={ref}
    className={cn('text-sm text-muted-foreground leading-relaxed', className)}
    {...props}
  />
))
AlertDescription.displayName = 'AlertDescription'

const inlineAlertVariants = cva(
  'inline-flex items-center gap-2 text-sm font-medium',
  {
    variants: {
      variant: {
        default: 'bg-dark-100 text-foreground',
        info: 'bg-green-light-100 text-green-light-900',
        destructive: 'bg-red-100 text-red',
        warning: 'bg-yellow-100 text-yellow-900',
        success: 'bg-green-dark-100 text-green-dark-900',
      },
    },
    defaultVariants: {
      variant: 'destructive',
    },
  },
)

export interface InlineAlertProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof inlineAlertVariants> {
  description: string
}

const InlineAlert = React.forwardRef<HTMLDivElement, InlineAlertProps>(
  ({ description, variant = 'destructive', className, ...props }, ref) => {
    const IconComponent = variantIcons[variant ?? 'destructive']
    return (
      <Alert
        ref={ref}
        variant={variant}
        size="compact"
        className={cn(inlineAlertVariants({ variant }), className)}
        {...props}
      >
        <IconComponent className="size-5 shrink-0" aria-hidden="true" />
        <span>{description}</span>
      </Alert>
    )
  },
)
InlineAlert.displayName = 'InlineAlert'

export {
  Alert,
  AlertIcon,
  AlertContent,
  AlertTitle,
  AlertDescription,
  InlineAlert,
  alertVariants,
  alertIconVariants,
}
