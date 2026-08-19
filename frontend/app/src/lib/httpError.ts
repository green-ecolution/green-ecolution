/** HTTP status of a rejected backend call; undefined when the error carries none. */
export const statusOf = (error: unknown): number | undefined =>
  (error as { response?: { status?: number } } | null)?.response?.status
