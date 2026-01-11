import * as React from 'react'
import { Upload, X, FileIcon } from 'lucide-react'

import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'

export interface FileUploadProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'onChange' | 'value'> {
  label?: string
  description?: string
  error?: string
  value?: File | null
  onChange?: (file: File | null) => void
  onClear?: () => void
}

const FileUpload = React.forwardRef<HTMLInputElement, FileUploadProps>(
  (
    {
      className,
      label,
      description,
      error,
      value,
      onChange,
      onClear,
      accept,
      id,
      ...props
    },
    ref
  ) => {
    const inputId = id || React.useId()
    const inputRef = React.useRef<HTMLInputElement>(null)

    React.useImperativeHandle(ref, () => inputRef.current!)

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0] || null
      onChange?.(file)
    }

    const handleClear = () => {
      if (inputRef.current) {
        inputRef.current.value = ''
      }
      onChange?.(null)
      onClear?.()
    }

    const handleClick = () => {
      inputRef.current?.click()
    }

    return (
      <div className={cn('space-y-2', className)}>
        {label && (
          <Label htmlFor={inputId} className={cn(error && 'text-destructive')}>
            {label}
            {props.required && <span className="text-destructive ml-1">*</span>}
          </Label>
        )}

        <div
          className={cn(
            'relative flex flex-col items-center justify-center rounded-lg border-2 border-dashed p-6 transition-colors',
            'hover:border-primary/50 hover:bg-accent/50',
            error ? 'border-destructive' : 'border-input',
            value && 'border-solid border-primary bg-primary/5'
          )}
        >
          <input
            ref={inputRef}
            type="file"
            id={inputId}
            accept={accept}
            onChange={handleChange}
            className="sr-only"
            aria-invalid={!!error}
            aria-describedby={error ? `${inputId}-error` : undefined}
            {...props}
          />

          {value ? (
            <div className="flex items-center gap-3">
              <FileIcon className="h-8 w-8 text-primary" />
              <div className="flex flex-col">
                <span className="text-sm font-medium">{value.name}</span>
                <span className="text-xs text-muted-foreground">
                  {(value.size / 1024).toFixed(1)} KB
                </span>
              </div>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                onClick={handleClear}
                className="ml-2"
              >
                <X className="h-4 w-4" />
                <span className="sr-only">Datei entfernen</span>
              </Button>
            </div>
          ) : (
            <button
              type="button"
              onClick={handleClick}
              className="flex flex-col items-center gap-2 text-center"
            >
              <Upload className="h-8 w-8 text-muted-foreground" />
              <span className="text-sm font-medium">Klicken zum Hochladen</span>
              {description && (
                <span className="text-xs text-muted-foreground">{description}</span>
              )}
            </button>
          )}
        </div>

        {error && (
          <p id={`${inputId}-error`} className="text-sm text-destructive">
            {error}
          </p>
        )}
      </div>
    )
  }
)
FileUpload.displayName = 'FileUpload'

export { FileUpload }
