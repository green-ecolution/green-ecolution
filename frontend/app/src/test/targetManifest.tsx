import type { ModuleManifest, ModuleTarget, SelectedTarget } from '@/modules/types'

export const probeSelection = (module: string, kind: string): SelectedTarget => ({
  target: { module, kind, id: `${kind}-1` },
  label: `${kind} 1`,
  detail: `${kind} detail`,
})

/** A sensor target whose slots render only buttons and test ids, so core tests see which slot it uses. */
export function probeTarget(
  module: string,
  kind: string,
  overrides: Partial<ModuleTarget> = {},
): ModuleTarget {
  return {
    kind,
    labelKey: 'common:modules.irrigation',
    invalidate: [[`${kind}-root`]],
    AssignDialog: ({ open, mode, onConfirm }) =>
      open ? (
        <button type="button" onClick={() => onConfirm(probeSelection(module, kind))}>
          {`${mode}-${kind}`}
        </button>
      ) : null,
    WizardStep: ({ onSelect, current }) => (
      <div>
        <p data-testid={`wizard-step-${kind}`}>{current?.id ?? 'none'}</p>
        <button type="button" onClick={() => onSelect(probeSelection(module, kind))}>
          {`pick-${kind}`}
        </button>
      </div>
    ),
    Placement: ({ sensor }) => <div data-testid={`placement-${kind}`}>{sensor.id}</div>,
    ...overrides,
  }
}

export function probeTargetManifest(key: string, targets: ModuleTarget[]): ModuleManifest {
  return {
    key,
    labelKey: 'common:modules.irrigation',
    i18n: { namespaces: [], load: () => Promise.resolve({}) },
    targets,
  }
}
