import { subtreeMemberCount, type OrgNode } from './organizationTree'

const personNoun = (count: number): string => (count === 1 ? 'Person' : 'Personen')

const subOrgNoun = (count: number): string =>
  count === 1 ? 'Unterorganisation' : 'Unterorganisationen'

/**
 * Subtitle of an organization row, in the tree as well as in the child list of
 * the detail panel. Only the headcount: the rows are narrow, and the disclosure
 * chevron already signals that a node has children. Null keeps the row to a
 * single line when the whole subtree is empty.
 */
export const memberSubtitle = (node: OrgNode): string | null => {
  const people = subtreeMemberCount(node)
  if (people === 0) return null
  return `${people} ${personNoun(people)}`
}

/** The headcount as a sentence, for the detail panel's member card. */
export const memberSummary = (node: OrgNode): string => {
  const people = subtreeMemberCount(node)
  const children = node.children.length
  if (people === 0) return 'Keine Personen zugeordnet'
  if (children === 0) return `${people} ${personNoun(people)}`
  return `${people} ${personNoun(people)} in ${children} ${subOrgNoun(children)}`
}

export const directChildrenLine = (count: number): string => `${count} direkte ${subOrgNoun(count)}`
