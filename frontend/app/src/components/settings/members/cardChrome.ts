export const CARD = '@container rounded-xl border border-dark-50 bg-white p-5 shadow-cards'
export const CARD_TITLE = 'font-lato text-base font-semibold text-dark'
export const TILE = 'flex shrink-0 items-center justify-center rounded-lg font-semibold'

export const sinceLabel = (createdAt?: string | null): string | null => {
  if (createdAt == null) return null
  const date = new Date(createdAt)
  if (Number.isNaN(date.getTime())) return null
  return `${String(date.getMonth() + 1).padStart(2, '0')}/${date.getFullYear()}`
}
