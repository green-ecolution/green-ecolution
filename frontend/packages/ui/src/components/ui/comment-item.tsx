import * as React from 'react'

import { cn } from '@/lib/utils'
import { useUiText } from '@/i18n'
import { Button } from './button'
import { Label } from './label'
import { Spinner } from './spinner'
import { Textarea } from './textarea'
import { CommentAuthorAvatar, type CommentAuthor } from './comment-composer'

export interface CommentItemProps extends React.HTMLAttributes<HTMLDivElement> {
  author: CommentAuthor
  body: string
  /** Already formatted by the caller. */
  timestamp: string
  editedLabel?: string
  canEdit?: boolean
  canDelete?: boolean
  onEdit?: (body: string) => void | Promise<void>
  onDelete?: () => void | Promise<void>
  isSaving?: boolean
}

const CommentItem = React.forwardRef<HTMLDivElement, CommentItemProps>(
  (
    {
      author,
      body,
      timestamp,
      editedLabel,
      canEdit = false,
      canDelete = false,
      onEdit,
      onDelete,
      isSaving = false,
      className,
      ...props
    },
    ref,
  ) => {
    const { t } = useUiText()
    const [isEditing, setIsEditing] = React.useState(false)
    const [editValue, setEditValue] = React.useState(body)
    const textareaId = React.useId()
    const textareaRef = React.useRef<HTMLTextAreaElement>(null)
    const editTriggerRef = React.useRef<HTMLButtonElement>(null)
    const wasEditingRef = React.useRef(false)

    // Effect (not the click handler) owns focus so it always runs after the
    // corresponding element has actually mounted.
    React.useEffect(() => {
      if (isEditing) {
        textareaRef.current?.focus()
        textareaRef.current?.select()
        wasEditingRef.current = true
      } else if (wasEditingRef.current) {
        wasEditingRef.current = false
        editTriggerRef.current?.focus()
      }
    }, [isEditing])

    const openEdit = () => {
      setEditValue(body)
      setIsEditing(true)
    }

    const handleCancel = () => {
      setEditValue(body)
      setIsEditing(false)
    }

    const trimmedEdit = editValue.trim()
    const canSave = trimmedEdit.length > 0 && !isSaving

    const handleSave = async () => {
      if (!canSave) return
      if (trimmedEdit === body) {
        setIsEditing(false)
        return
      }
      try {
        await onEdit?.(trimmedEdit)
        setIsEditing(false)
      } catch {
        // stay in edit mode so the author can retry
      }
    }

    return (
      <div ref={ref} data-slot="comment-item" className={cn('flex gap-3', className)} {...props}>
        <CommentAuthorAvatar author={author} size="sm" />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-baseline gap-x-2">
            <span className="text-sm font-semibold text-dark-800">{author.name}</span>
            <span className="text-xs text-dark-500">{timestamp}</span>
            {editedLabel && <span className="text-xs italic text-dark-400">{editedLabel}</span>}
          </div>

          {isEditing ? (
            <div className="mt-2 flex flex-col gap-2">
              <Label htmlFor={textareaId} className="sr-only">
                {t('comments.edit')}
              </Label>
              <Textarea
                id={textareaId}
                ref={textareaRef}
                value={editValue}
                onChange={(event) => setEditValue(event.target.value)}
                disabled={isSaving}
                rows={3}
              />
              <div className="flex justify-end gap-2">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={handleCancel}
                  disabled={isSaving}
                >
                  {t('comments.cancel')}
                </Button>
                <Button type="button" size="sm" onClick={handleSave} disabled={!canSave}>
                  {isSaving && <Spinner className="size-4" />}
                  {t('comments.save')}
                </Button>
              </div>
            </div>
          ) : (
            <>
              <p className="mt-1 whitespace-pre-wrap break-words text-sm text-dark-800">{body}</p>
              {(canEdit || canDelete) && (
                <div className="mt-1 flex gap-3">
                  {canEdit && (
                    <Button
                      ref={editTriggerRef}
                      type="button"
                      variant="link"
                      size="sm"
                      className="h-auto px-0"
                      onClick={openEdit}
                    >
                      {t('comments.edit')}
                    </Button>
                  )}
                  {canDelete && (
                    <Button
                      type="button"
                      variant="link-destructive"
                      size="sm"
                      className="h-auto px-0"
                      onClick={() => onDelete?.()}
                    >
                      {t('comments.delete')}
                    </Button>
                  )}
                </div>
              )}
            </>
          )}
        </div>
      </div>
    )
  },
)
CommentItem.displayName = 'CommentItem'

export { CommentItem }
