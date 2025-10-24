import { SoilCondition } from '@green-ecolution/backend-client'

export const SoilConditionOptions = [
  {
    value: SoilCondition.TreeSoilConditionSchluffig,
    label: 'Schluffig',
  },
  {
    value: SoilCondition.TreeSoilConditionSandig,
    label: 'Sandig',
  },
  {
    value: SoilCondition.TreeSoilConditionLehmig,
    label: 'Lehmig',
  },
  {
    value: SoilCondition.TreeSoilConditionTonig,
    label: 'Tonig',
  },
  {
    value: SoilCondition.TreeSoilConditionUnknown,
    label: 'Unbekannt',
  },
]
