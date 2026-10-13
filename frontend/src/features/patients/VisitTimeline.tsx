import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import { CalendarX2, FileText, Lock, RotateCcw, Stethoscope } from 'lucide-react'
import type { Report, VisitSummary } from '../../api/types'
import { VisitReports } from '../reports/VisitReports'
import { StatusBadge } from '../../components/StatusBadge'
import { fmtDate } from '../../utils/date'
import { medicineName } from '../prescription/medicine'

/** The patient's visits as a vertical timeline, newest first (§5.4 Visits tab). */
export function VisitTimeline({
  visits,
  canOpen,
  onRepeat,
  reports = [],
  showValues = false,
}: {
  visits: VisitSummary[]
  canOpen: boolean
  /** The patient's uploaded reports; each visit card shows its own. */
  reports?: Report[]
  /** Doctors see the values read from each report (§4). */
  showValues?: boolean
  /** Repeat ℞ (PM-10): start today's prescription with this visit's medicines. */
  onRepeat?: (visit: VisitSummary) => void
}) {
  const { t } = useTranslation()
  if (visits.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-10 text-center text-slate-500">
        <span className="flex h-12 w-12 items-center justify-center rounded-2xl bg-slate-100 text-slate-400">
          <CalendarX2 className="h-6 w-6" />
        </span>
        {t('visits.none')}
      </div>
    )
  }

  return (
    <ol className="relative space-y-4 before:absolute before:top-2 before:bottom-2 before:left-[11px] before:w-px before:bg-slate-200">
      {visits.map((v) => {
        const rx = v.prescription
        return (
          <li key={v.id} className="relative flex gap-4">
            <span
              className={`relative z-10 mt-1 flex h-6 w-6 shrink-0 items-center justify-center rounded-full ring-4 ring-white ${
                rx ? 'bg-brand text-white' : v.status === 'cancelled' ? 'bg-slate-200 text-slate-400' : 'bg-brand-100 text-brand'
              }`}
            >
              {rx ? <FileText className="h-3 w-3" /> : <Stethoscope className="h-3 w-3" />}
            </span>
            <div className="min-w-0 flex-1 rounded-xl border border-slate-200 bg-white p-3.5 transition-shadow hover:shadow-md">
              <div className="flex flex-wrap items-center gap-2 text-sm">
                <span className="font-semibold text-slate-900">{fmtDate(v.visit_date)}</span>
                <span className="text-slate-500">
                  {t('visits.visitNo', { n: v.visit_no })} · {t(`visits.type.${v.visit_type}`)}
                </span>
                <StatusBadge status={v.status} />
                {rx && (
                  <span className="inline-flex items-center gap-1 text-xs text-slate-500">
                    {rx.locked && <Lock className="h-3 w-3" />}℞ v{rx.version}
                    {!rx.locked && ` · ${t('rx.draft')}`}
                  </span>
                )}
                <span className="ml-auto flex flex-wrap gap-1.5">
                  {canOpen && v.status !== 'cancelled' && (
                    <Link
                      to={`/visits/${v.id}/prescription`}
                      className="inline-flex items-center gap-1 rounded-lg border border-brand/30 px-2.5 py-1 text-xs font-medium text-brand hover:bg-brand-50"
                    >
                      <FileText className="h-3.5 w-3.5" />
                      {rx ? t('visits.viewRx') : t('visits.startRx')}
                    </Link>
                  )}
                  {onRepeat && (rx?.medicines.length ?? 0) > 0 && (
                    <button
                      onClick={() => onRepeat(v)}
                      className="inline-flex items-center gap-1 rounded-lg border border-slate-200 px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50"
                    >
                      <RotateCcw className="h-3.5 w-3.5" />
                      {t('visits.repeat')}
                    </button>
                  )}
                </span>
              </div>
              {(() => {
                const mine = reports.filter((r) => r.visit_id === v.id)
                return mine.length > 0 ? (
                  <div className="mt-3 border-t border-slate-100 pt-3">
                    <VisitReports reports={mine} showValues={showValues} />
                  </div>
                ) : null
              })()}
              {rx && (rx.diagnosis.length > 0 || rx.complaints.length > 0 || rx.medicines.length > 0) && (
                <div className="mt-2 space-y-1.5 text-sm">
                  {rx.diagnosis.length > 0 && <div className="font-medium text-slate-900">{rx.diagnosis.join(', ')}</div>}
                  {rx.complaints.length > 0 && <div className="text-slate-500">{rx.complaints.join(', ')}</div>}
                  {rx.medicines.length > 0 && (
                    <div className="flex flex-wrap gap-1.5">
                      {rx.medicines.map((m, i) => (
                        <span key={i} className="rounded-md bg-slate-100 px-2 py-0.5 text-xs text-slate-700">
                          ℞ {medicineName(m)}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          </li>
        )
      })}
    </ol>
  )
}
