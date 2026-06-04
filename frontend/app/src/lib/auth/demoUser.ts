export interface DemoProfile {
  preferred_username: string
  email: string
  given_name: string
  family_name: string
  driving_licenses: string[]
  user_roles: string[]
  status: string
}

export const DEMO_PROFILE: DemoProfile = {
  preferred_username: 'ttester',
  email: 'toni.tester@green-ecolution.de',
  given_name: 'Toni',
  family_name: 'Tester',
  driving_licenses: ['B', 'BE', 'C', 'CE'],
  user_roles: ['green-ecolution'],
  status: 'available',
}

function base64Url(json: unknown): string {
  return btoa(JSON.stringify(json)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

// JWT-shaped so decodeJWT reads the middle segment; signature is irrelevant
// because the backend skips verification when auth.enabled = false.
export const DEMO_ACCESS_TOKEN = `header.${base64Url(DEMO_PROFILE)}.sig`
