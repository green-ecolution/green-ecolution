import { SoilCondition } from '@green-ecolution/backend-client'

export interface SoilConditionOption {
  value: SoilCondition
  label: string
  group: string
}

// KA5 fine soil types, grouped by Hauptbodenart.
export const SoilConditionOptions: SoilConditionOption[] = [
  { value: SoilCondition.Ss, label: 'Ss – Reinsand', group: 'Sande' },
  { value: SoilCondition.Su2, label: 'Su2 – schwach schluffiger Sand', group: 'Sande' },
  { value: SoilCondition.Su3, label: 'Su3 – schluffiger Sand', group: 'Sande' },
  { value: SoilCondition.Su4, label: 'Su4 – stark schluffiger Sand', group: 'Sande' },
  { value: SoilCondition.Sl2, label: 'Sl2 – schwach lehmiger Sand', group: 'Sande' },
  { value: SoilCondition.Sl3, label: 'Sl3 – lehmiger Sand', group: 'Sande' },
  { value: SoilCondition.Sl4, label: 'Sl4 – stark lehmiger Sand', group: 'Sande' },
  { value: SoilCondition.Slu, label: 'Slu – schluffig-lehmiger Sand', group: 'Sande' },
  { value: SoilCondition.St2, label: 'St2 – schwach toniger Sand', group: 'Sande' },
  { value: SoilCondition.St3, label: 'St3 – toniger Sand', group: 'Sande' },
  { value: SoilCondition.Uu, label: 'Uu – reiner Schluff', group: 'Schluffe' },
  { value: SoilCondition.Us, label: 'Us – sandiger Schluff', group: 'Schluffe' },
  { value: SoilCondition.Uls, label: 'Uls – sandig-lehmiger Schluff', group: 'Schluffe' },
  { value: SoilCondition.Ut2, label: 'Ut2 – schwach toniger Schluff', group: 'Schluffe' },
  { value: SoilCondition.Ut3, label: 'Ut3 – toniger Schluff', group: 'Schluffe' },
  { value: SoilCondition.Ut4, label: 'Ut4 – stark toniger Schluff', group: 'Schluffe' },
  { value: SoilCondition.Ls2, label: 'Ls2 – schwach sandiger Lehm', group: 'Lehme' },
  { value: SoilCondition.Ls3, label: 'Ls3 – sandiger Lehm', group: 'Lehme' },
  { value: SoilCondition.Ls4, label: 'Ls4 – stark sandiger Lehm', group: 'Lehme' },
  { value: SoilCondition.Lt2, label: 'Lt2 – schwach toniger Lehm', group: 'Lehme' },
  { value: SoilCondition.Lt3, label: 'Lt3 – toniger Lehm', group: 'Lehme' },
  { value: SoilCondition.Lts, label: 'Lts – sandig-toniger Lehm', group: 'Lehme' },
  { value: SoilCondition.Lu, label: 'Lu – schluffiger Lehm', group: 'Lehme' },
  { value: SoilCondition.Tt, label: 'Tt – reiner Ton', group: 'Tone' },
  { value: SoilCondition.Tu2, label: 'Tu2 – schwach schluffiger Ton', group: 'Tone' },
  { value: SoilCondition.Tu3, label: 'Tu3 – schluffiger Ton', group: 'Tone' },
  { value: SoilCondition.Tu4, label: 'Tu4 – stark schluffiger Ton', group: 'Tone' },
  { value: SoilCondition.Ts2, label: 'Ts2 – schwach sandiger Ton', group: 'Tone' },
  { value: SoilCondition.Ts3, label: 'Ts3 – sandiger Ton', group: 'Tone' },
  { value: SoilCondition.Ts4, label: 'Ts4 – stark sandiger Ton', group: 'Tone' },
  { value: SoilCondition.Tl, label: 'Tl – lehmiger Ton', group: 'Tone' },
  { value: SoilCondition.FS, label: 'fS – Feinsand', group: 'Reinsande' },
  { value: SoilCondition.MS, label: 'mS – Mittelsand', group: 'Reinsande' },
  { value: SoilCondition.GS, label: 'gS – Grobsand', group: 'Reinsande' },
  { value: SoilCondition.Unknown, label: 'Unbekannt', group: 'Sonstige' },
]

export const soilConditionLabel = (value: SoilCondition): string =>
  SoilConditionOptions.find((o) => o.value === value)?.label ?? value
