export interface KeycloakJWT {
  jti: string
  exp: number
  nbf: number
  iat: number
  iss: string
  aud: string
  sub: string
  typ: string
  azp: string
  session_state: string
  acr: string
  email_verified: boolean
  name: string
  preferred_username: string
  given_name: string
  family_name: string
  email: string
  driving_licenses: string[]
  user_roles: string[]
  status: string
}
