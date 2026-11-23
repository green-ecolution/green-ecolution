import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  translateIssue,
  validateEmail,
  validateUsername,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import { useIssueTranslator } from '@/lib/i18n/validation'

const USERNAME_MAX_LENGTH = 64

// wasm-bindgen emits `(value: string) => any`; tighten the return type at the boundary.
const emailValidator = validateEmail as (value: string) => ValidationIssue | null
const usernameValidator = validateUsername as (value: string) => ValidationIssue | null

export type AddMemberField = 'firstName' | 'lastName' | 'email' | 'username'

export interface AddMemberValues extends Record<AddMemberField, string> {
  roleIds: string[]
}

const EMPTY: AddMemberValues = {
  firstName: '',
  lastName: '',
  email: '',
  username: '',
  roleIds: [],
}

export const usernameFromEmail = (email: string): string =>
  email.split('@')[0].toLowerCase().replace(/\s/g, '').slice(0, USERNAME_MAX_LENGTH)

export const useAddMemberForm = () => {
  const { t } = useTranslation('settings')
  const translate = useIssueTranslator()
  const [values, setValues] = useState<AddMemberValues>(EMPTY)
  const [usernameEdited, setUsernameEdited] = useState(false)
  const [touched, setTouched] = useState<ReadonlySet<AddMemberField>>(() => new Set())

  const setField = <K extends AddMemberField | 'roleIds'>(field: K, value: AddMemberValues[K]) => {
    setValues((current) => {
      const next = { ...current, [field]: value }
      if (field === 'email' && !usernameEdited) next.username = usernameFromEmail(value as string)
      return next
    })
    if (field === 'username') setUsernameEdited(true)
    if (field !== 'roleIds') setTouched((current) => new Set(current).add(field))
  }

  const reset = () => {
    setValues(EMPTY)
    setUsernameEdited(false)
    setTouched(new Set())
  }

  const translated = (issue: ValidationIssue | null): string | null =>
    issue ? translateIssue(issue, translate) : null

  const issues: Record<AddMemberField, string | null> = {
    firstName: values.firstName.trim() === '' ? t('members.add.firstNameRequired') : null,
    lastName: values.lastName.trim() === '' ? t('members.add.lastNameRequired') : null,
    email: translated(emailValidator(values.email.trim())),
    username: translated(usernameValidator(values.username.trim())),
  }

  const errors: Partial<Record<AddMemberField, string>> = {}
  for (const field of touched) {
    const message = issues[field]
    if (message) errors[field] = message
  }

  const valid = Object.values(issues).every((message) => message === null)

  return { values, errors, valid, setField, reset }
}
