/** Two-letter monogram for avatar tiles. Empty when no name is known. */
export const initialsOf = (firstName?: string | null, lastName?: string | null): string =>
  `${firstName?.charAt(0) ?? ''}${lastName?.charAt(0) ?? ''}`.toUpperCase()

/** Monogram for entities that carry a single name, e.g. an organization tile. */
export const initialsOfName = (name: string): string =>
  name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((word) => word[0]?.toUpperCase() ?? '')
    .join('')
