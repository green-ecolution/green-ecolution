import type { BadgeProps } from '@green-ecolution/ui'

export type StatusColor = NonNullable<BadgeProps['variant']>

// `t`'s generated overloads only accept the catalog's literal key union; the
// enum value plugged into the template isn't statically one of those literals.
export type EnumsTranslate = (key: string) => string
