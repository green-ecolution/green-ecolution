import * as React from 'react'

import { cn } from '@/lib/utils'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'

export interface FormFieldProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label: string
  error?: string
  description?: string
  hideLabel?: boolean
}

const FormField = React.forwardRef<HTMLInputElement, FormFieldProps>(
  ({ className, label, error, description, hideLabel, id, ...props }, ref) => {
    const inputId = id || React.useId()

    return (
      <div data-slot="form-field" className={cn('space-y-2', className)}>
        <Label
          htmlFor={inputId}
          className={cn(hideLabel && 'sr-only', error && 'text-destructive')}
        >
          {label}
          {props.required && <span className="text-destructive ml-1">*</span>}
        </Label>
        <Input
          ref={ref}
          id={inputId}
          aria-invalid={!!error}
          aria-describedby={
            error ? `${inputId}-error` : description ? `${inputId}-description` : undefined
          }
          className={cn(error && 'border-destructive focus-visible:ring-destructive')}
          {...props}
        />
        {description && !error && (
          <p
            id={`${inputId}-description`}
            data-slot="form-field-description"
            className="text-sm text-muted-foreground"
          >
            {description}
          </p>
        )}
        {error && (
          <p
            id={`${inputId}-error`}
            data-slot="form-field-error"
            className="text-sm text-destructive"
          >
            {error}
          </p>
        )}
      </div>
    )
  },
)
FormField.displayName = 'FormField'

export interface TextareaFieldProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label: string
  error?: string
  description?: string
  hideLabel?: boolean
}

const TextareaField = React.forwardRef<HTMLTextAreaElement, TextareaFieldProps>(
  ({ className, label, error, description, hideLabel, id, ...props }, ref) => {
    const inputId = id || React.useId()

    return (
      <div data-slot="textarea-field" className={cn('space-y-2', className)}>
        <Label
          htmlFor={inputId}
          className={cn(hideLabel && 'sr-only', error && 'text-destructive')}
        >
          {label}
          {props.required && <span className="text-destructive ml-1">*</span>}
        </Label>
        <Textarea
          ref={ref}
          id={inputId}
          aria-invalid={!!error}
          aria-describedby={
            error ? `${inputId}-error` : description ? `${inputId}-description` : undefined
          }
          className={cn(error && 'border-destructive focus-visible:ring-destructive')}
          {...props}
        />
        {description && !error && (
          <p
            id={`${inputId}-description`}
            data-slot="textarea-field-description"
            className="text-sm text-muted-foreground"
          >
            {description}
          </p>
        )}
        {error && (
          <p
            id={`${inputId}-error`}
            data-slot="textarea-field-error"
            className="text-sm text-destructive"
          >
            {error}
          </p>
        )}
      </div>
    )
  },
)
TextareaField.displayName = 'TextareaField'

export { FormField, TextareaField }
