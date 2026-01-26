import type { FieldErrors, FieldValues, Resolver } from 'react-hook-form'
import type { z } from 'zod'

type ZodSchema<T> = z.ZodType<T>

interface ResolverSuccess<TFieldValues extends FieldValues> {
  values: TFieldValues
  errors: Record<string, never>
}

interface ResolverError<TFieldValues extends FieldValues> {
  values: Record<string, never>
  errors: FieldErrors<TFieldValues>
}

/**
 * Custom zodResolver wrapper for Zod v4 compatibility with react-hook-form.
 *
 * This resolver handles the type incompatibilities between Zod v4 and
 * @hookform/resolvers by implementing the resolver interface directly.
 */
export function zodResolver<TOutput extends FieldValues>(
  schema: ZodSchema<TOutput>,
): Resolver<TOutput> {
  return async (values) => {
    const result = await schema.safeParseAsync(values)

    if (result.success) {
      return {
        values: result.data,
        errors: {},
      } as ResolverSuccess<TOutput>
    }

    const errors: FieldErrors<TOutput> = {}

    for (const issue of result.error.issues) {
      const path = issue.path.join('.')
      if (path && !errors[path as keyof TOutput]) {
        errors[path as keyof TOutput] = {
          type: issue.code,
          message: issue.message,
        } as FieldErrors<TOutput>[keyof TOutput]
      }
    }

    return {
      values: {},
      errors,
    } as ResolverError<TOutput>
  }
}
