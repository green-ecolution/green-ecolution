import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Combobox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  FormField,
  Label,
  SelectField,
  TextareaField,
} from '@green-ecolution/ui'
import type { OrganizationResponse, PluginFrontendDto } from '@/api/backendApi'
import PluginPermissionMatrix from './PluginPermissionMatrix'
import { usePluginPermissionDraft } from './usePluginPermissionDraft'
import {
  buildFrontendDto,
  frontendModeOptions,
  validateTarget,
  type FrontendMode,
} from './pluginFrontend'
import { validateSlug } from './pluginSlug'

export type { FrontendMode } from './pluginFrontend'

export interface PluginInstallPayload {
  slug: string
  name: string
  description: string | null
  organizationId: string
  permissions: string[]
  requiredPermissions: string[]
  frontend: PluginFrontendDto
}

interface PluginInstallDialogProps {
  open: boolean
  organizations?: OrganizationResponse[]
  saving?: boolean
  onOpenChange: (open: boolean) => void
  onSubmit: (payload: PluginInstallPayload) => void
}

interface FieldErrors {
  slug?: string | null
  name?: string | null
  organization?: string | null
  target?: string | null
}

export const PluginInstallDialog = ({
  open,
  organizations = [],
  saving = false,
  onOpenChange,
  onSubmit,
}: PluginInstallDialogProps) => {
  const { t } = useTranslation(['settings', 'common'])

  const [slug, setSlug] = useState('')
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [organizationId, setOrganizationId] = useState('')
  const [frontendMode, setFrontendMode] = useState<FrontendMode>('none')
  const [target, setTarget] = useState('')
  const [errors, setErrors] = useState<FieldErrors>({})
  const {
    permissions,
    accessPermissions,
    togglePermission,
    toggleAccessPermission,
    reset: resetPermissionDraft,
  } = usePluginPermissionDraft()

  const reset = () => {
    setSlug('')
    setName('')
    setDescription('')
    setOrganizationId('')
    setFrontendMode('none')
    setTarget('')
    resetPermissionDraft()
    setErrors({})
  }

  const handleOpenChange = (next: boolean) => {
    if (!next) reset()
    onOpenChange(next)
  }

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault()

    const slugTrimmed = slug.trim()
    const nameTrimmed = name.trim()
    const descriptionTrimmed = description.trim()

    const nextErrors: FieldErrors = {
      slug: validateSlug(slugTrimmed, t),
      name: nameTrimmed === '' ? t('plugin.install.nameRequired') : null,
      organization: organizationId === '' ? t('plugin.install.organizationRequired') : null,
      target: validateTarget(frontendMode, target, t),
    }
    setErrors(nextErrors)

    if (nextErrors.slug || nextErrors.name || nextErrors.organization || nextErrors.target) return
    if (saving) return

    onSubmit({
      slug: slugTrimmed,
      name: nameTrimmed,
      description: descriptionTrimmed === '' ? null : descriptionTrimmed,
      organizationId,
      permissions: [...permissions],
      requiredPermissions: [...accessPermissions],
      frontend: buildFrontendDto(frontendMode, target),
    })
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      {/* Two rows — header, then a body that takes whatever height is left —
          so the panel never grows past the viewport and the form scrolls
          inside it instead of the page. */}
      <DialogContent className="max-h-[calc(100dvh-3rem)] max-w-2xl grid-rows-[auto_minmax(0,1fr)] overflow-hidden">
        <DialogHeader>
          <DialogTitle>{t('plugin.install.title')}</DialogTitle>
          <DialogDescription>{t('plugin.install.description')}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="flex min-h-0 flex-col">
          {/* The scroll region spans the panel's full width and re-applies the
              padding inside, so focus rings are not clipped at its edges. */}
          <div className="-mx-6 flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-6 py-1">
            <FormField
              id="plugin-install-slug"
              label={t('plugin.install.slugLabel')}
              value={slug}
              onChange={(event) => setSlug(event.target.value)}
              error={errors.slug ?? undefined}
              description={errors.slug ? undefined : t('plugin.install.slugHint')}
              required
            />

            <FormField
              id="plugin-install-name"
              label={t('plugin.install.nameLabel')}
              value={name}
              onChange={(event) => setName(event.target.value)}
              error={errors.name ?? undefined}
              required
            />

            <TextareaField
              id="plugin-install-description"
              label={t('plugin.install.descriptionLabel')}
              value={description}
              onChange={(event) => setDescription(event.target.value)}
            />

            <div className="flex flex-col gap-y-2">
              <Label htmlFor="plugin-install-organization">
                {t('plugin.install.organizationLabel')}
                <span className="text-destructive ml-1">*</span>
              </Label>
              <Combobox
                id="plugin-install-organization"
                options={organizations.map((org) => ({ value: org.id, label: org.name }))}
                value={organizationId}
                onChange={setOrganizationId}
                placeholder={t('plugin.install.organizationPlaceholder')}
                searchPlaceholder={t('plugin.install.organizationSearchPlaceholder')}
                aria-invalid={!!errors.organization}
              />
              {errors.organization && (
                <p role="alert" aria-live="assertive" className="text-sm text-destructive">
                  {errors.organization}
                </p>
              )}
            </div>

            <SelectField
              id="plugin-install-frontend-mode"
              label={t('plugin.install.frontendModeLabel')}
              value={frontendMode}
              onValueChange={(value) => setFrontendMode(value as FrontendMode)}
              options={frontendModeOptions(t)}
            />

            {frontendMode !== 'none' && (
              <FormField
                id="plugin-install-target"
                label={t('plugin.install.targetLabel')}
                value={target}
                onChange={(event) => setTarget(event.target.value)}
                error={errors.target ?? undefined}
                description={
                  errors.target
                    ? undefined
                    : frontendMode === 'external'
                      ? t('plugin.install.targetExternalHint')
                      : t('plugin.install.targetProxiedHint')
                }
                placeholder={
                  frontendMode === 'external' ? 'https://plugin.example.com' : 'plugin-backend:8080'
                }
                required
              />
            )}

            <PluginPermissionMatrix
              heading={t('plugin.install.permissionsHeading')}
              hint={t('plugin.install.permissionsHint')}
              permissions={permissions}
              onToggle={togglePermission}
            />

            <PluginPermissionMatrix
              heading={t('plugin.install.accessPermissionsHeading')}
              hint={t('plugin.install.accessPermissionsHint')}
              permissions={accessPermissions}
              onToggle={toggleAccessPermission}
            />
          </div>

          {/* Outside the scroll region: the submit button must stay reachable
              however long the permission matrices make the form. */}
          <DialogFooter className="-mx-6 -mb-6 mt-4 rounded-b-xl border-t border-dark-100 px-6 py-4">
            <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
              {t('common:actions.cancel')}
            </Button>
            <Button type="submit" disabled={saving}>
              {t('plugin.install.submit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default PluginInstallDialog
