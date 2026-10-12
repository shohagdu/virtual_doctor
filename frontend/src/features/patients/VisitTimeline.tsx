import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { VisitSummary } from '../../api/types'
import { StatusBadge } from '../../components/StatusBadge'
import { fmtDate } from '../../utils/date'
import { medicineName } from '../prescription/medicine'

export function VisitTimeline({
  visits,
  canOpen,
  onRepeat,
}: {
  visits: VisitSummary[]
  canOpen: boolean
  /** Repeat ℞ (PM-10): start today's prescription with this visit's medicines. */
  onRepeat?: (visit: VisitSummary) => void
}) {
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
              <StatusBadge status={v.status} />
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
                {v.prescription.medicines.length > 0 && <div>℞ {v.prescription.medicines.map(medicineName).join(', ')}</div>}
              </>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            {canOpen && v.status !== 'cancelled' && (
              <Link to={`/visits/${v.id}/prescription`} className="rounded border px-3 py-1 text-sm text-brand">
                {v.prescription ? t('visits.viewRx') : t('visits.startRx')}
              </Link>
            )}
            {onRepeat && (v.prescription?.medicines.length ?? 0) > 0 && (
              <button onClick={() => onRepeat(v)} className="rounded border px-3 py-1 text-sm text-brand">
                ↻ {t('visits.repeat')}
              </button>
            )}
          </div>
        </li>
      ))}
    </ol>
  )
}
