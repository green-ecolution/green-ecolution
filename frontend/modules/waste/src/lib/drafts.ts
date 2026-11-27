import {
  draftResolver,
  wasteValidateBinDraft,
  type IssueTranslator,
} from '@green-ecolution/domain-wasm'
import type { FieldValues, Resolver } from 'react-hook-form'

export interface WasteBinForm {
  number: string
  latitude: number
  longitude: number
  volumeLiters: number
  depthCm: number
  /** Owning organization. Only sent on create; the backend defaults to the caller's org. */
  organizationId?: string
}

export const wasteBinDraftResolver = <TForm extends FieldValues>(
  translate: IssueTranslator,
): Resolver<TForm> => draftResolver<TForm>(wasteValidateBinDraft, translate)
