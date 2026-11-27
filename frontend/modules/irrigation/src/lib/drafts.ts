import {
  draftResolver,
  irrigationValidateTreeClusterDraft,
  irrigationValidateTreeDraft,
  type IssueTranslator,
} from '@green-ecolution/domain-wasm'
import type { FieldValues, Resolver } from 'react-hook-form'

export const treeDraftResolver = <TForm extends FieldValues>(
  translate: IssueTranslator,
): Resolver<TForm> => draftResolver<TForm>(irrigationValidateTreeDraft, translate)

export const clusterDraftResolver = <TForm extends FieldValues>(
  translate: IssueTranslator,
): Resolver<TForm> => draftResolver<TForm>(irrigationValidateTreeClusterDraft, translate)
