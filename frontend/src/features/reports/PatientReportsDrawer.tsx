import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FileText, FlaskConical, ImageIcon, Maximize2, X } from 'lucide-react'
import type { Report } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { usePatientReports } from './api'
import { ReportViewer } from './ReportViewer'
import { isImage, reportTitle, useReportFile } from './reportFile'

const dateOf = (r: Report) => r.report_date ?? r.created_at.slice(0, 10)

/**
 * All of the patient's uploaded reports, from the ℞ screen: a thumbnail gallery
 * grouped by date (this visit's first); click one to open it in the viewer.
 */
export function PatientReportsDrawer({
  patientId,
  visitId,
  showValues,
  onClose,
}: {
  patientId: number
  visitId: number
  showValues: boolean
  onClose: () => void
}) {
  const { t } = useTranslation()
  const { data, isLoading } = usePatientReports(String(patientId))
  const [open, setOpen] = useState<number | null>(null)

  // This visit's reports first, then the rest newest first; the viewer steps through them in this order.
  const reports = [...(data ?? [])].sort((a, b) => {
    const mine = Number(b.visit_id === visitId) - Number(a.visit_id === visitId)
    return mine || dateOf(b).localeCompare(dateOf(a)) || b.id - a.id
  })
  const groups: { key: string; label: string; items: { r: Report; i: number }[] }[] = []
  reports.forEach((r, i) => {
    const current = r.visit_id === visitId
    const key = current ? 'this' : dateOf(r)
    let g = groups.find((x) => x.key === key)
    if (!g) {
      g = { key, label: current ? t('reports.thisVisit') : fmtDate(dateOf(r)), items: [] }
      groups.push(g)
    }
    g.items.push({ r, i })
  })

  return (
    <div className="fixed inset-0 z-30 print:hidden" role="dialog" aria-label={t('reports.patientReports')}>
      <div className="absolute inset-0 bg-slate-900/40 backdrop-blur-[2px]" onClick={onClose} />
      <aside className="absolute inset-y-0 right-0 flex w-full max-w-2xl flex-col bg-white shadow-2xl">
        <div className="flex items-center gap-2 border-b px-5 py-4">
          <span className="flex h-9 w-9 items-center justify-center rounded-xl bg-brand-50 text-brand">
            <FlaskConical className="h-5 w-5" />
          </span>
          <div>
            <h2 className="font-semibold text-slate-900">{t('reports.patientReports')}</h2>
            <p className="text-xs text-slate-500">{t('reports.nReports', { count: reports.length })}</p>
          </div>
          <button onClick={onClose} className="ml-auto rounded-lg p-2 hover:bg-slate-100" aria-label={t('common.close')}>
            <X className="h-5 w-5" />
          </button>
        </div>
        <div className="flex-1 space-y-6 overflow-y-auto p-5">
          {isLoading ? (
            <p className="text-slate-500">{t('common.loading')}</p>
          ) : reports.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-16 text-center text-slate-500">
              <span className="flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100 text-slate-400">
                <FlaskConical className="h-7 w-7" />
              </span>
              {t('reports.none')}
            </div>
          ) : (
            groups.map((g) => (
              <section key={g.key} className="space-y-2">
                <h3 className={`text-xs font-semibold tracking-wide uppercase ${g.key === 'this' ? 'text-brand' : 'text-slate-500'}`}>
                  {g.label} · {g.items.length}
                </h3>
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
                  {g.items.map(({ r, i }) => (
                    <GalleryCard key={r.id} r={r} onOpen={() => setOpen(i)} />
                  ))}
                </div>
              </section>
            ))
          )}
        </div>
      </aside>
      {open !== null && (
        <ReportViewer reports={reports} index={open} onIndex={setOpen} onClose={() => setOpen(null)} showValues={showValues} />
      )}
    </div>
  )
}

function GalleryCard({ r, onOpen }: { r: Report; onOpen: () => void }) {
  const { t } = useTranslation()
  const file = useReportFile(r, isImage(r))
  const flagged = r.results.filter((x) => x.status && x.status !== 'normal')
  const critical = flagged.some((x) => x.status === 'critical')
  return (
    <button
      type="button"
      onClick={onOpen}
      className="group overflow-hidden rounded-xl border border-slate-200 bg-white text-left hover:border-brand/40 hover:shadow-md"
      title={t('reports.open')}
    >
      <span className="relative flex aspect-[4/3] items-center justify-center overflow-hidden bg-slate-100 text-slate-400">
        {isImage(r) ? (
          file.data ? (
            <img src={file.data} alt="" className="h-full w-full object-cover transition-transform group-hover:scale-105" />
          ) : (
            <ImageIcon className="h-8 w-8" />
          )
        ) : (
          <FileText className="h-10 w-10 text-red-500" />
        )}
        <span className="absolute inset-0 flex items-center justify-center bg-slate-900/40 text-white opacity-0 transition-opacity group-hover:opacity-100">
          <Maximize2 className="h-6 w-6" />
        </span>
        {flagged.length > 0 && (
          <span
            className={`absolute top-2 left-2 rounded-full px-2 py-0.5 text-[11px] font-semibold text-white ${critical ? 'bg-red-600' : 'bg-orange-500'}`}
          >
            {critical ? '⚠ ' : ''}
            {flagged.length} {t('reports.flagged')}
          </span>
        )}
      </span>
      <span className="block px-3 py-2">
        <span className="block truncate text-sm font-medium text-slate-800">{reportTitle(r, t('reports.untitled'))}</span>
        <span className="block truncate text-xs text-slate-500">
          {fmtDate(dateOf(r))} · {t(`reports.state.${r.status}`)}
        </span>
      </span>
    </button>
  )
}
