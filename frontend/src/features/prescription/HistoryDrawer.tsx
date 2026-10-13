import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { MedicineLine, VisitSummary } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { medicineDirections, medicineName } from './medicine'

/** 🕘 Previous visits, slid in over the ℞ screen (§5.5 history drawer). */
export function HistoryDrawer({
  visits,
  onCopy,
  onClose,
}: {
  visits: VisitSummary[]
  onCopy: (medicines: MedicineLine[]) => void
  onClose: () => void
}) {
  const { t } = useTranslation()
  return (
    <div className="fixed inset-0 z-30 print:hidden" role="dialog" aria-label={t('rx.previousVisits')}>
      <div className="absolute inset-0 bg-slate-900/40 backdrop-blur-[2px]" onClick={onClose} />
      <aside className="absolute inset-y-0 right-0 flex w-full max-w-md flex-col bg-white shadow-2xl">
        <div className="flex items-center border-b px-4 py-3">
          <h2 className="font-semibold">🕘 {t('rx.previousVisits')}</h2>
          <button onClick={onClose} className="ml-auto px-2 text-lg" aria-label={t('common.close')}>
            ✕
          </button>
        </div>
        <div className="flex-1 divide-y overflow-y-auto">
          {visits.length === 0 && <p className="p-4 text-sm text-slate-500">{t('visits.none')}</p>}
          {visits.map((v) => (
            <PastVisit key={v.id} visit={v} onCopy={onCopy} />
          ))}
        </div>
      </aside>
    </div>
  )
}

function PastVisit({ visit: v, onCopy }: { visit: VisitSummary; onCopy: (m: MedicineLine[]) => void }) {
  const { t } = useTranslation()
  const meds = v.prescription?.medicines ?? []
  // All medicines selected by default; untick to copy only some (PM-10).
  const [picked, setPicked] = useState<boolean[]>(() => meds.map(() => true))
  const chosen = meds.filter((_, i) => picked[i])

  return (
    <section className="space-y-2 p-4 text-sm">
      <div>
        <b>{fmtDate(v.visit_date)}</b>
        {v.prescription && v.prescription.diagnosis.length > 0 && <> — {v.prescription.diagnosis.join(', ')}</>}
        <span className="ml-2 text-xs text-slate-500">{t('visits.visitNo', { n: v.visit_no })}</span>
      </div>
      {v.prescription && v.prescription.complaints.length > 0 && (
        <div className="text-slate-500">{v.prescription.complaints.join(', ')}</div>
      )}
      {meds.length > 0 ? (
        <ul className="space-y-1">
          {meds.map((m, i) => (
            <li key={i}>
              <label className="flex gap-2">
                <input
                  type="checkbox"
                  checked={picked[i]}
                  onChange={(e) => setPicked(picked.map((p, j) => (j === i ? e.target.checked : p)))}
                  className="mt-1"
                />
                <span>
                  ℞ <b>{medicineName(m)}</b> {medicineDirections(m)}
                </span>
              </label>
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-slate-400">{t('rx.noMedicines')}</p>
      )}
      <div className="flex flex-wrap gap-2">
        {meds.length > 0 && (
          <button
            onClick={() => onCopy(chosen)}
            disabled={chosen.length === 0}
            className="rounded-md bg-brand px-2 py-1 text-xs text-white disabled:opacity-50"
          >
            {chosen.length === meds.length ? t('rx.copyMedicines') : t('rx.copySelected', { n: chosen.length })}
          </button>
        )}
        {v.prescription && (
          <a
            href={`/visits/${v.id}/prescription`}
            target="_blank"
            rel="noreferrer"
            className="rounded-md border px-2 py-1 text-xs text-brand"
          >
            {t('visits.viewRx')} ↗
          </a>
        )}
      </div>
    </section>
  )
}
