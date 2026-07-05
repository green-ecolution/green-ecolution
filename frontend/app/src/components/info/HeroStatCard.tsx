import { Card, CardContent, cn } from '@green-ecolution/ui'

interface HeroStatCardProps {
  /** Gradient overlay classes, e.g. 'bg-gradient-to-br from-green-dark/5 to-transparent' */
  gradient: string
  icon: React.ReactNode
  /** Icon box classes besides 'rounded-xl', e.g. 'p-3 bg-green-dark/10' */
  iconBoxClassName: string
  className?: string
  headerClassName?: string
  /** Rendered below the header row (links, detail grids, ...) */
  footer?: React.ReactNode
  children: React.ReactNode
}

const HeroStatCard = ({
  gradient,
  icon,
  iconBoxClassName,
  className,
  headerClassName,
  footer,
  children,
}: HeroStatCardProps) => (
  <Card className={cn('relative overflow-hidden', className)}>
    <div className={cn('absolute inset-0', gradient)} />
    <CardContent className="pt-6 relative">
      <div className={cn('flex items-start justify-between', headerClassName)}>
        {children}
        <div className={cn('rounded-xl', iconBoxClassName)}>{icon}</div>
      </div>
      {footer}
    </CardContent>
  </Card>
)

export default HeroStatCard
