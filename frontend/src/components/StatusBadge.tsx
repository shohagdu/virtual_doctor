import { useTranslation } from 'react-i18next'
import type { VisitStatus } from '../api/types'

const statusClass: Record<VisitStatus, string> = {
  waiting: 'bg-amber-100 text-amber-800',
  in_consultation: 'bg-blue-100 text-blue-800',
  seen: 'bg-green-100 text-green-800',
  cancelled: 'bg-slate-200 text-slate-600',
}

/** Queue status colour badge (§5.1). */
export function StatusBadge({ status }: { status: VisitStatus }) {
  const { t } = useTranslation()
  return (
    <span className={`whitespace-nowrap rounded px-2 text-xs ${statusClass[status]}`}>
      {status === 'seen' ? '✔ ' : status === 'waiting' ? '● ' : ''}
      {t(`visits.status.${status}`)}
    </span>
  )
}
