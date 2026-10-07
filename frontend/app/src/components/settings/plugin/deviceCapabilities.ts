export const CAPABILITIES = ['camera', 'bluetooth'] as const

export const toggledCapability = (
  current: ReadonlySet<string>,
  capability: string,
): ReadonlySet<string> => {
  const next = new Set(current)
  if (next.has(capability)) next.delete(capability)
  else next.add(capability)
  return next
}

/** Fixed order, so a request does not depend on the order of the clicks. */
export const orderedCapabilities = (capabilities: ReadonlySet<string>): string[] =>
  CAPABILITIES.filter((capability) => capabilities.has(capability))
