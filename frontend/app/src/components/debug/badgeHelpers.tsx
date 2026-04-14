import { Badge } from '@green-ecolution/ui'

export const boolBadge = (value: boolean) =>
  value ? <Badge variant="success">true</Badge> : <Badge variant="error">false</Badge>
