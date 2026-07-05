/** Builds a finder over `options`, falling back to `fallback` (default: first option). */
export const createEnumLookup =
  <V, O extends { value: V }>(options: readonly O[], fallback: O = options[0]) =>
  (value: V): O =>
    options.find((option) => option.value === value) ?? fallback

/** Builds a parser that maps raw strings to enum values via `mapping`, else `fallback`. */
export const createEnumParser =
  <V>(mapping: Record<string, V>, fallback: V) =>
  (raw: string): V =>
    Object.prototype.hasOwnProperty.call(mapping, raw) ? mapping[raw] : fallback
