import * as React from 'react'
import type { VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'
import { useUiText } from '@/i18n'
import { Avatar, AvatarFallback, AvatarImage, avatarVariants } from './avatar'
import { Button } from './button'
import { Label } from './label'
import { Spinner } from './spinner'
import { Textarea } from './textarea'

export interface CommentAuthor {
  name: string
  avatarUrl?: string | null
}

function initialsFromName(name: string): string {
  return name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((word) => word[0]?.toUpperCase() ?? '')
    .join('')
}

interface CommentAuthorAvatarProps {
  author: CommentAuthor
  size?: VariantProps<typeof avatarVariants>['size']
}

function CommentAuthorAvatar({ author, size = 'default' }: CommentAuthorAvatarProps) {
  return (
    <Avatar size={size}>
      {author.avatarUrl && <AvatarImage src={author.avatarUrl} alt={author.name} />}
      <AvatarFallback variant="user">{initialsFromName(author.name)}</AvatarFallback>
    </Avatar>
  )
}

export interface CommentComposerProps extends Omit<
  React.HTMLAttributes<HTMLDivElement>,
  'onSubmit'
> {
  author: CommentAuthor
  onSubmit: (body: string) => void | Promise<void>
  isSubmitting?: boolean
  /** Mirrors the domain's `CommentBody` limit. */
  maxLength?: number
  placeholder?: string
  disabled?: boolean
}

const CommentComposer = React.forwardRef<HTMLDivElement, CommentComposerProps>(
  (
    {
      author,
      onSubmit,
      isSubmitting = false,
      maxLength = 2000,
      placeholder,
      disabled = false,
      className,
      ...props
    },
    ref,
  ) => {
    const { t } = useUiText()
    const [value, setValue] = React.useState('')
    const textareaId = React.useId()

    const trimmed = value.trim()
    const canSubmit = trimmed.length > 0 && !disabled && !isSubmitting
    const showCounter = value.length > maxLength * 0.8
    const remaining = Math.max(0, maxLength - value.length)
    const label = placeholder ?? t('comments.placeholder')

    const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      if (!canSubmit) return
      try {
        await onSubmit(trimmed)
        setValue('')
      } catch {
        // keep the draft so the author doesn't lose it on a failed submit
      }
    }

    return (
      <div
        ref={ref}
        data-slot="comment-composer"
        className={cn('flex gap-3', className)}
        {...props}
      >
        <CommentAuthorAvatar author={author} size="default" />
        <form onSubmit={handleSubmit} className="flex flex-1 flex-col gap-2">
          <Label htmlFor={textareaId} className="sr-only">
            {label}
          </Label>
          <Textarea
            id={textareaId}
            value={value}
            onChange={(event) => setValue(event.target.value)}
            placeholder={label}
            maxLength={maxLength}
            disabled={disabled || isSubmitting}
            rows={3}
          />
          <div aria-live="polite" className="min-h-[1rem] text-right text-xs text-dark-500">
            {showCounter && <span>{t('comments.charactersLeft', { count: remaining })}</span>}
          </div>
          <div className="flex justify-end">
            <Button type="submit" size="sm" disabled={!canSubmit}>
              {isSubmitting && <Spinner className="size-4" />}
              {t('comments.submit')}
            </Button>
          </div>
        </form>
      </div>
    )
  },
)
CommentComposer.displayName = 'CommentComposer'

export { CommentComposer, CommentAuthorAvatar, initialsFromName }
