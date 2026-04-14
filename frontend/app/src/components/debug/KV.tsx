const KV = ({ label, children }: { label: string; children: React.ReactNode }) => (
  <div className="flex items-start justify-between gap-3">
    <span className="text-muted-foreground shrink-0">{label}</span>
    <span className="text-right">{children}</span>
  </div>
)

export default KV
