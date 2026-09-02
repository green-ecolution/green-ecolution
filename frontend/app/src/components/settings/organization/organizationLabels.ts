import type { TFunction } from 'i18next'
import { subtreeMemberCount, type OrgNode } from './organizationTree'

/**
 * Subtitle of an organization row, in the tree as well as in the child list of
 * the detail panel. Only the headcount: the rows are narrow, and the disclosure
 * chevron already signals that a node has children. Null keeps the row to a
 * single line when the whole subtree is empty.
 */
export const memberSubtitle = (node: OrgNode, t: TFunction<'settings'>): string | null => {
  const people = subtreeMemberCount(node)
  if (people === 0) return null
  return t('organization.count', { count: people })
}

/** The headcount as a sentence, for the detail panel's member card. */
export const memberSummary = (node: OrgNode, t: TFunction<'settings'>): string => {
  const people = subtreeMemberCount(node)
  const children = node.children.length
  if (people === 0) return t('organization.noMembersAssigned')
  const peopleLabel = t('organization.count', { count: people })
  if (children === 0) return peopleLabel
  return t('organization.memberSummaryWithChildren', {
    people: peopleLabel,
    children: t('organization.subOrganizationCount', { count: children }),
  })
}

export const directChildrenLine = (count: number, t: TFunction<'settings'>): string =>
  t('organization.directChildrenLine', {
    count,
    noun: t('organization.subOrganizationNoun', { count }),
  })
