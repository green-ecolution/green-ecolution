/** Every key the Rust validators of this module can emit; the texts live under `waste:validation`. */
export const WASTE_VALIDATION_KEYS = [
  'waste_bin.number.empty',
  'waste_bin.number.tooLong',
  'waste_bin.volume.outOfRange',
  'waste_bin.volume.invalidFormat',
  'waste_bin.depth.outOfRange',
  'waste_bin.depth.invalidFormat',
  'settings.full_threshold.outOfRange',
  'settings.full_threshold.invalidFormat',
] as const satisfies readonly string[]
