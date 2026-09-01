/** Builds a parser that maps raw strings to enum values via `mapping`, else `fallback`. */
export const createEnumParser =
  <V>(mapping: Record<string, V>, fallback: V) =>
  (raw: string): V =>
    Object.prototype.hasOwnProperty.call(mapping, raw) ? mapping[raw] : fallback
