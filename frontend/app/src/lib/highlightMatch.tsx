import type { ReactNode } from 'react'

const escapeRegExp = (value: string) => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

export const highlightMatch = (text: string, query: string): ReactNode => {
  const needle = query.trim()
  if (needle.length === 0) return text

  const parts = text.split(new RegExp(`(${escapeRegExp(needle)})`, 'gi'))
  return parts.map((part, index) =>
    part.toLowerCase() === needle.toLowerCase() ? (
      // eslint-disable-next-line react-x/no-array-index-key -- split() parts have no stable id and the array is regenerated whenever text or query change
      <mark key={index} className="bg-yellow-200 text-dark-900">
        {part}
      </mark>
    ) : (
      part
    ),
  )
}
