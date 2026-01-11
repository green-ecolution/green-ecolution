import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { TriangleAlert } from 'lucide-react'

import { cn } from '@/lib/utils'

const alertVariants = cva('relative rounded-xl border border-dark-50 bg-white shadow-cards', {
  variants: {
    variant: {
      default: 'bg-white text-foreground',
      destructive: 'border-red-100 bg-red-50 [&>svg]:text-destructive',
      warning: 'border-yellow-100 bg-yellow-50 [&>svg]:text-yellow',
      success: 'border-green-dark-100 bg-green-dark-50 [&>svg]:text-green-dark',
    },
    size: {
      default:
        'w-full p-6 [&>svg~*]:pl-8 [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-6 [&>svg]:top-6 [&>svg]:h-5 [&>svg]:w-5 [&>svg]:text-muted-foreground',
      compact:
        'w-fit p-4 flex items-center gap-x-2 rounded-2xl border-0 shadow-none [&>svg]:h-6 [&>svg]:w-6 [&>svg]:flex-shrink-0',
    },
  },
  compoundVariants: [
    {
      variant: 'destructive',
      size: 'compact',
      className:
        'bg-red-100 [&>svg]:text-red [&>p]:text-red [&>p]:text-sm [&>p]:font-semibold [&>p]:ml-2',
    },
  ],
  defaultVariants: {
    variant: 'default',
    size: 'default',
  },
})

const Alert = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & VariantProps<typeof alertVariants>
>(({ className, variant, size, ...props }, ref) => (
  <div
    ref={ref}
    role="alert"
    className={cn(alertVariants({ variant, size }), className)}
    {...props}
  />
))
Alert.displayName = 'Alert'

const AlertTitle = React.forwardRef<HTMLParagraphElement, React.HTMLAttributes<HTMLHeadingElement>>(
  ({ className, ...props }, ref) => (
    <h5
      ref={ref}
      className={cn(
        'mb-1 font-lato text-lg font-semibold leading-tight tracking-tight text-foreground',
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
  <div ref={ref} className={cn('text-base [&_p]:leading-relaxed', className)} {...props} />
))
AlertDescription.displayName = 'AlertDescription'

export interface InlineAlertProps extends React.HTMLAttributes<HTMLDivElement> {
  description: string
}

const InlineAlert = React.forwardRef<HTMLDivElement, InlineAlertProps>(
  ({ description, className, ...props }, ref) => (
    <Alert ref={ref} variant="destructive" size="compact" className={className} {...props}>
      <TriangleAlert />
      <p>{description}</p>
    </Alert>
  ),
)
InlineAlert.displayName = 'InlineAlert'

export { Alert, AlertTitle, AlertDescription, InlineAlert, alertVariants }
