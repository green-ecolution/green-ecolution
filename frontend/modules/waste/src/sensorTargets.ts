import { lazy } from 'react'
import type { ModuleTarget } from '@/modules/types'
import { wasteRoots } from './api/roots'
import { BIN_KIND } from './lib/targets'

// Lazy for the same reason as the map and planning slots: the manifest sits in
// the core bundle next to the registry, and these components reach back into
// core modules that import that registry.
export const binSensorTarget: ModuleTarget = {
  kind: BIN_KIND,
  labelKey: 'waste:sensor.target.bin',
  texts: {
    activateAction: 'waste:sensor.binTexts.activateAction',
    reassignAction: 'waste:sensor.binTexts.reassignAction',
    removeLinkAction: 'waste:sensor.binTexts.removeLinkAction',
    removeConfirmTitle: 'waste:sensor.binTexts.removeConfirmTitle',
    removeConfirmDescription: 'waste:sensor.binTexts.removeConfirmDescription',
    activateSuccess: 'waste:sensor.binTexts.activateSuccess',
    reassignSuccess: 'waste:sensor.binTexts.reassignSuccess',
    activateConflict: 'waste:sensor.binTexts.activateConflict',
    reassignNotFound: 'waste:sensor.binTexts.reassignNotFound',
    reassignConflict: 'waste:sensor.binTexts.reassignConflict',
    reassignFailed: 'waste:sensor.binTexts.reassignFailed',
    scanDescription: 'waste:sensor.binTexts.scanDescription',
    alreadyLinkedDescription: 'waste:sensor.binTexts.alreadyLinkedDescription',
  },
  invalidate: wasteRoots.bin,
  AssignDialog: lazy(() => import('./components/sensor/SensorBinAssignDialog')),
  WizardStep: lazy(() => import('./components/sensor/wizard/SensorBinStep')),
  Placement: lazy(() => import('./components/sensor/BinPlacement')),
}
