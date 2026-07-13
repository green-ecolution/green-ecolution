import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'
import { Separator } from '@/components/ui/separator'

const buttonGroupVariants = cva(
  'flex w-fit items-stretch [&>*]:focus-visible:z-10 [&>*]:focus-visible:relative [&>input]:flex-1 has-[>[data-slot=button-group]]:gap-2',
  {
    variants: {
      orientation: {
        horizontal:
          '[&>*:not(:first-child)]:rounded-l-none [&>*:not(:first-child)]:border-l-0 [&>*:not(:last-child)]:rounded-r-none',
        vertical:
          'flex-col [&>*:not(:first-child)]:rounded-t-none [&>*:not(:first-child)]:border-t-0 [&>*:not(:last-child)]:rounded-b-none',
      },
    },
    defaultVariants: {
      orientation: 'horizontal',
    },
  },
)

const ButtonGroup = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & VariantProps<typeof buttonGroupVariants>
>(({ className, orientation, ...props }, ref) => (
  <div
    ref={ref}
    role="group"
    data-slot="button-group"
    data-orientation={orientation ?? 'horizontal'}
    className={cn(buttonGroupVariants({ orientation }), className)}
    {...props}
  />
))
ButtonGroup.displayName = 'ButtonGroup'

const ButtonGroupText = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      'flex items-center gap-2 rounded-lg border border-dark-600 bg-muted px-4 text-base font-medium [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0',
      className,
    )}
    {...props}
  />
))
ButtonGroupText.displayName = 'ButtonGroupText'

const ButtonGroupSeparator = React.forwardRef<
  React.ComponentRef<typeof Separator>,
  React.ComponentPropsWithoutRef<typeof Separator>
>(({ className, orientation = 'vertical', ...props }, ref) => (
  <Separator
    ref={ref}
    orientation={orientation}
    className={cn('relative !m-0 self-stretch data-[orientation=vertical]:h-auto', className)}
    {...props}
  />
))
ButtonGroupSeparator.displayName = 'ButtonGroupSeparator'

export { ButtonGroup, ButtonGroupSeparator, ButtonGroupText, buttonGroupVariants }
