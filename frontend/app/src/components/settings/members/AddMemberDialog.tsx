import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  CopyableText,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  FormField,
  Label,
  MultiSelectCombobox,
} from '@green-ecolution/ui'
import type { RoleResponse, UserResponse } from '@/api/backendApi'
import { resolveApiError, type ApiErrorInfo } from '@/lib/apiError'
import createToast from '@/hooks/createToast'
import { useUserMutations } from '@/hooks/useUserMutations'
import { useAddMemberForm } from './addMemberForm'
import { fullNameOf } from './memberList'

const IDENTITY_TAKEN = 'user.identity_taken'

/** A cause the backend named beats the generic failure text. */
const namesCause = ({ messageKey }: ApiErrorInfo) =>
  messageKey.startsWith('code.') || messageKey.startsWith('validation:')

interface AddMemberDialogProps {
  open: boolean
  organizationId: string
  organizationName: string
  /** Null where the viewer may not read roles: the field is left out. */
  roles: RoleResponse[] | null
  onOpenChange: (open: boolean) => void
  onCreated: (user: UserResponse) => void
  /** True from submitting until the result is closed: leaving then loses the password. */
  onBusyChange?: (busy: boolean) => void
}

interface CreatedPerson {
  user: UserResponse
  initialPassword: string
}

const AddMemberDialog = ({
  open,
  organizationId,
  organizationName,
  roles,
  onOpenChange,
  onCreated,
  onBusyChange,
}: AddMemberDialogProps) => {
  const { t } = useTranslation(['settings', 'common'])
  const { createUser } = useUserMutations()
  const showToast = createToast()
  const form = useAddMemberForm()
  const { values, errors, valid, setField } = form
  // The initial password lives here and nowhere else; closing discards it.
  const [created, setCreated] = useState<CreatedPerson | null>(null)
  const [identityTaken, setIdentityTaken] = useState(false)
  const [rolesTooPowerful, setRolesTooPowerful] = useState(false)

  const close = () => {
    const person = created
    setCreated(null)
    setIdentityTaken(false)
    setRolesTooPowerful(false)
    form.reset()
    createUser.reset()
    onBusyChange?.(false)
    onOpenChange(false)
    if (person) onCreated(person.user)
  }

  const handleOpenChange = (next: boolean) => {
    if (next) onOpenChange(true)
    // Closing now would detach the request and lose the one-time password.
    else if (!createUser.isPending) close()
  }

  const edit: typeof setField = (field, value) => {
    setIdentityTaken(false)
    setRolesTooPowerful(false)
    setField(field, value)
  }

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault()
    // Disabling the button alone doesn't stop Enter-to-submit in every browser.
    if (!valid || createUser.isPending) return
    setIdentityTaken(false)
    setRolesTooPowerful(false)
    const roleIds = roles === null ? [] : values.roleIds
    onBusyChange?.(true)
    createUser.mutate(
      {
        organizationId,
        firstName: values.firstName.trim(),
        lastName: values.lastName.trim(),
        email: values.email.trim(),
        username: values.username.trim(),
        roleIds,
      },
      {
        onSuccess: ({ user, initialPassword }) => {
          setCreated({ user, initialPassword })
          createUser.reset()
        },
        onError: (error) => {
          onBusyChange?.(false)
          void resolveApiError(error).then((info) => {
            if (info.code === IDENTITY_TAKEN) setIdentityTaken(true)
            // The only 403 a permitted caller can meet: a role beyond their own grants.
            else if (info.status === 403 && roleIds.length > 0) setRolesTooPowerful(true)
            else showToast(namesCause(info) ? info.message : t('members.add.failed'), 'error')
          })
        },
      },
    )
  }

  const roleOptions = (roles ?? [])
    .filter((role) => !role.isTemplate)
    .map((role) => ({ value: role.id, label: role.name }))

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent
        onInteractOutside={(event) => {
          if (created || createUser.isPending) event.preventDefault()
        }}
        onEscapeKeyDown={(event) => {
          // The password cannot be shown again, so only "Fertig" or the close button leave it.
          if (created || createUser.isPending) event.preventDefault()
        }}
      >
        {created ? (
          <>
            <DialogHeader>
              <DialogTitle>
                {t('members.add.createdTitle', { name: fullNameOf(created.user) })}
              </DialogTitle>
              <DialogDescription>{t('members.add.passwordHint')}</DialogDescription>
            </DialogHeader>
            <CopyableText label={t('members.add.passwordLabel')} value={created.initialPassword} />
            <DialogFooter>
              <Button type="button" onClick={close}>
                {t('members.add.done')}
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>{t('members.add.title')}</DialogTitle>
              <DialogDescription>
                {t('members.add.description', { name: organizationName })}
              </DialogDescription>
            </DialogHeader>

            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <div className="grid gap-4 sm:grid-cols-2">
                <FormField
                  id="add-member-first-name"
                  label={t('members.add.firstName')}
                  value={values.firstName}
                  onChange={(e) => edit('firstName', e.target.value)}
                  error={errors.firstName}
                  autoComplete="off"
                />
                <FormField
                  id="add-member-last-name"
                  label={t('members.add.lastName')}
                  value={values.lastName}
                  onChange={(e) => edit('lastName', e.target.value)}
                  error={errors.lastName}
                  autoComplete="off"
                />
              </div>

              {identityTaken && (
                <p role="alert" className="text-sm text-destructive">
                  {t('members.add.identityTaken')}
                </p>
              )}
              <FormField
                id="add-member-email"
                type="email"
                label={t('members.add.email')}
                value={values.email}
                onChange={(e) => edit('email', e.target.value)}
                error={errors.email}
                autoComplete="off"
              />
              <FormField
                id="add-member-username"
                label={t('members.add.username')}
                description={t('members.add.usernameHint')}
                value={values.username}
                onChange={(e) => edit('username', e.target.value)}
                error={errors.username}
                autoComplete="off"
              />

              {roles !== null && (
                <div className="flex flex-col gap-y-2">
                  <Label htmlFor="add-member-roles">{t('members.add.roles')}</Label>
                  <MultiSelectCombobox
                    id="add-member-roles"
                    options={roleOptions}
                    value={values.roleIds}
                    onChange={(ids) => edit('roleIds', ids)}
                  />
                  {rolesTooPowerful && (
                    <p role="alert" className="text-sm text-destructive">
                      {t('members.roleErrorTooPowerful')}
                    </p>
                  )}
                  <p className="text-sm text-muted-foreground">{t('members.add.rolesHint')}</p>
                </div>
              )}

              <DialogFooter>
                <Button
                  type="button"
                  variant="outline"
                  disabled={createUser.isPending}
                  onClick={close}
                >
                  {t('common:actions.cancel')}
                </Button>
                <Button type="submit" disabled={!valid || createUser.isPending}>
                  {t('members.add.submit')}
                </Button>
              </DialogFooter>
            </form>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

export default AddMemberDialog
export type { AddMemberDialogProps }
