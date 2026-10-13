import { useTranslation } from 'react-i18next'
import type { VisitStatus } from '../api/types'

const statusClass: Record<VisitStatus, string> = {
  waiting: 'bg-amber-50 text-amber-800 ring-amber-200',
  in_consultation: 'bg-brand-50 text-brand ring-brand/20',
  seen: 'bg-emerald-50 text-emerald-700 ring-emerald-200',
  cancelled: 'bg-slate-100 text-slate-500 ring-slate-200',
}

const dotClass: Record<VisitStatus, string> = {
  waiting: 'bg-amber-500',
  in_consultation: 'bg-brand animate-pulse',
  seen: 'bg-emerald-500',
  cancelled: 'bg-slate-400',
}

/** Queue status colour badge (§5.1). */
export function StatusBadge({ status }: { status: VisitStatus }) {
  const { t } = useTranslation()
  return (
    <span className={`inline-flex items-center gap-1.5 whitespace-nowrap rounded-full px-2 py-0.5 text-xs font-medium ring-1 ${statusClass[status]}`}>
      <span className={`h-1.5 w-1.5 rounded-full ${dotClass[status]}`} />
      {t(`visits.status.${status}`)}
    </span>
  )
}
