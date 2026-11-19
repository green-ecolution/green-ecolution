import type { TFunction } from 'i18next'
import { subtreeMemberCount, type OrgNode } from './organizationTree'

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
