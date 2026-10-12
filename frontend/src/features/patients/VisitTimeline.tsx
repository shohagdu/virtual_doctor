import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { VisitSummary } from '../../api/types'
import { fmtDate } from '../../utils/date'

const statusClass: Record<VisitSummary['status'], string> = {
  waiting: 'bg-amber-100 text-amber-800',
  in_consultation: 'bg-blue-100 text-blue-800',
  seen: 'bg-green-100 text-green-800',
  cancelled: 'bg-slate-200 text-slate-600',
}

export function VisitTimeline({ visits, canOpen }: { visits: VisitSummary[]; canOpen: boolean }) {
  const { t } = useTranslation()
  if (visits.length === 0) return <p className="text-slate-500">{t('visits.none')}</p>

  return (
    <ol className="divide-y">
      {visits.map((v) => (
        <li key={v.id} className="flex flex-wrap items-start gap-3 py-3">
          <div className="min-w-0 flex-1 space-y-1 text-sm">
            <div className="flex flex-wrap items-center gap-2">
              <b>{fmtDate(v.visit_date)}</b>
              <span className="text-slate-500">
                {t('visits.visitNo', { n: v.visit_no })} · {t(`visits.type.${v.visit_type}`)}
              </span>
              <span className={`rounded px-2 text-xs ${statusClass[v.status]}`}>{t(`visits.status.${v.status}`)}</span>
              {v.prescription && (
                <span className="text-xs text-slate-500">
                  ℞ v{v.prescription.version}
                  {v.prescription.locked ? ' 🔒' : ` (${t('rx.draft')})`}
                </span>
              )}
            </div>
            {v.prescription && (
              <>
                {v.prescription.diagnosis.length > 0 && <div>{v.prescription.diagnosis.join(', ')}</div>}
                {v.prescription.complaints.length > 0 && (
                  <div className="text-slate-500">{v.prescription.complaints.join(', ')}</div>
                )}
                {v.prescription.medicines.length > 0 && <div>℞ {v.prescription.medicines.join(', ')}</div>}
              </>
            )}
          </div>
          {canOpen && v.status !== 'cancelled' && (
            <Link to={`/visits/${v.id}/prescription`} className="rounded border px-3 py-1 text-sm text-brand">
              {v.prescription ? t('visits.viewRx') : t('visits.startRx')}
            </Link>
          )}
        </li>
      ))}
    </ol>
  )
}
