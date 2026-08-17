/** Two-letter monogram for avatar tiles. Empty when no name is known. */
export const initialsOf = (firstName?: string | null, lastName?: string | null): string =>
  `${firstName?.charAt(0) ?? ''}${lastName?.charAt(0) ?? ''}`.toUpperCase()
