import * as React from 'react'
import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'
import { MoveRight } from 'lucide-react'

import { cn } from '../../lib/utils'

const linkCardVariants = cva(
  'shadow-cards border h-full p-6 rounded-xl group flex flex-col gap-4 transition-all ease-in-out duration-300',
  {
    variants: {
      variant: {
        dark: 'border-green-dark bg-green-dark-50 hover:bg-green-dark-100',
        light: 'border-green-light bg-green-light-50 hover:bg-green-light-100',
        white: 'border-dark-50 bg-white hover:bg-dark-100',
      },
    },
    defaultVariants: {
      variant: 'white',
    },
  },
)

export interface LinkCardProps
  extends React.HTMLAttributes<HTMLElement>,
    VariantProps<typeof linkCardVariants> {
  asChild?: boolean
}

const LinkCard = React.forwardRef<HTMLElement, LinkCardProps>(
  ({ className, variant, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'div'
    return (
      <Comp
        className={cn(linkCardVariants({ variant, className }))}
        ref={ref as React.Ref<HTMLDivElement>}
        {...props}
      />
    )
  },
)
LinkCard.displayName = 'LinkCard'

const LinkCardTitle = React.forwardRef<
  HTMLHeadingElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <h3
    ref={ref}
    className={cn('font-lato text-lg text-dark font-semibold', className)}
    {...props}
  />
))
LinkCardTitle.displayName = 'LinkCardTitle'

const LinkCardDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p ref={ref} className={cn('text-dark-800', className)} {...props} />
))
LinkCardDescription.displayName = 'LinkCardDescription'

export interface LinkCardFooterProps
  extends React.HTMLAttributes<HTMLParagraphElement> {
  showArrow?: boolean
}

const LinkCardFooter = React.forwardRef<HTMLParagraphElement, LinkCardFooterProps>(
  ({ className, children, showArrow = true, ...props }, ref) => (
    <p
      ref={ref}
      className={cn(
        'font-lato font-semibold text-green-dark flex items-center gap-x-2 mt-auto pt-2',
        className,
      )}
      {...props}
    >
      <span>{children}</span>
      {showArrow && (
        <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
      )}
    </p>
  ),
)
LinkCardFooter.displayName = 'LinkCardFooter'

export { LinkCard, LinkCardTitle, LinkCardDescription, LinkCardFooter, linkCardVariants }
