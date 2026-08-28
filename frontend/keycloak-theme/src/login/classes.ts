import type { ClassKey } from 'keycloakify/login'

export const classes = {
  kcHtmlClass: 'h-full',
  kcBodyClass: 'h-full',

  kcFormGroupClass: 'mb-5 flex flex-col gap-2',
  kcLabelClass: 'text-sm font-semibold text-foreground',
  kcInputClass:
    'w-full rounded-lg border border-input bg-card px-4 py-3 text-sm text-foreground outline-none transition-colors placeholder:text-muted-foreground/70 focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/30 aria-[invalid=true]:border-destructive',
  kcInputErrorMessageClass: 'text-sm text-destructive',
  kcInputGroup: 'relative flex items-center',
  kcFormPasswordVisibilityButtonClass:
    'absolute right-2 inline-flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40',

  kcButtonClass:
    'group inline-flex items-center justify-center gap-2 rounded-full px-5 py-3.5 text-sm font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background active:scale-[0.99] disabled:cursor-not-allowed disabled:opacity-60',
  kcButtonPrimaryClass: 'bg-primary text-primary-foreground hover:bg-primary/90',
  kcButtonDefaultClass: 'border border-input bg-background text-foreground hover:bg-accent',
  kcButtonSecondaryClass: 'border border-input bg-background text-foreground hover:bg-accent',
  kcButtonBlockClass: 'w-full',
  kcButtonLargeClass: 'py-3',

  kcAlertClass: 'mb-6 rounded-lg border px-4 py-3 text-sm',
  kcAlertTitleClass: 'block',

  kcFormSocialAccountSectionClass: 'mt-6',
  kcFormSocialAccountListClass: 'flex list-none flex-col gap-3 p-0',
  kcFormSocialAccountListButtonClass:
    'flex w-full items-center justify-center gap-2 rounded-full border border-input bg-card px-5 py-3 text-sm font-semibold text-foreground no-underline transition-colors hover:bg-accent',
  kcFormSocialAccountNameClass: 'truncate',

  kcFormOptionsClass: 'flex items-center',
  kcFormOptionsWrapperClass: 'ml-auto',
  kcFormSettingClass: 'flex flex-wrap items-center justify-between gap-3',
  kcCheckboxInputClass: 'h-4 w-4 rounded-sm accent-[var(--primary)]',

  kcSignUpClass: 'mt-6 text-center text-sm text-muted-foreground',
  kcInfoAreaWrapperClass: '',
  kcSrOnlyClass: 'sr-only',
  kcContentWrapperClass: '',
  kcFormClass: '',
} satisfies Partial<Record<ClassKey, string>>
