import * as React from 'react'

import { cn } from '@/lib/utils'
import { useUiText } from '@/i18n'

export interface CommentListProps extends React.HTMLAttributes<HTMLDivElement> {
  children?: React.ReactNode
  isLoading?: boolean
  emptyLabel?: string
}

function CommentListSkeletonRow() {
  return (
    <div className="flex gap-3" aria-hidden="true">
      <div className="size-8 shrink-0 animate-pulse rounded-full bg-dark-100" />
      <div className="flex flex-1 flex-col justify-center gap-2">
        <div className="h-3 w-1/4 animate-pulse rounded bg-dark-100" />
        <div className="h-3 w-2/3 animate-pulse rounded bg-dark-100" />
      </div>
    </div>
  )
}

const CommentList = React.forwardRef<HTMLDivElement, CommentListProps>(
  ({ children, isLoading = false, emptyLabel, className, ...props }, ref) => {
    const { t } = useUiText()
    const isEmpty = !isLoading && React.Children.count(children) === 0

    return (
      <div
        ref={ref}
        data-slot="comment-list"
        aria-busy={isLoading}
        className={cn('flex flex-col gap-4', className)}
        {...props}
      >
        {isLoading ? (
          <>
            <CommentListSkeletonRow />
            <CommentListSkeletonRow />
            <CommentListSkeletonRow />
          </>
        ) : isEmpty ? (
          <p className="py-6 text-center text-sm text-dark-500">
            {emptyLabel ?? t('comments.empty')}
          </p>
        ) : (
          children
        )}
      </div>
    )
  },
)
CommentList.displayName = 'CommentList'

export { CommentList }
