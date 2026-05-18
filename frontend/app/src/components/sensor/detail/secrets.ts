// LoRaWAN credentials that must NEVER be shown in plain text without an
// explicit reveal action. APPKEY is the OTAA root key; APPSKEY/NWKSKEY are
// session keys; PWORD is the device's AT-interface password.
const SENSITIVE_CONFIG_KEYS = new Set(['APPKEY', 'APPSKEY', 'NWKSKEY', 'PWORD'])

export const isSensitiveConfigKey = (key: string): boolean =>
  SENSITIVE_CONFIG_KEYS.has(key.toUpperCase())

export const SECRET_MASK = '••••••••••••'

// Replace sensitive values with a placeholder for safe clipboard exports.
export const redactConfig = (config: Record<string, unknown>): Record<string, unknown> => {
  const out: Record<string, unknown> = {}
  for (const [k, v] of Object.entries(config)) {
    out[k] = isSensitiveConfigKey(k) ? '<redacted>' : v
  }
  return out
}
