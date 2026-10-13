import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FileText, FlaskConical, ImageIcon } from 'lucide-react'
import type { Report } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { ReportViewer } from './ReportViewer'
import { isImage, reportTitle as title, useReportFile } from './reportFile'

/** Reports uploaded for one visit, as small cards with a thumbnail; click to view. */
export function VisitReports({ reports, showValues }: { reports: Report[]; showValues: boolean }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState<number | null>(null)
  if (reports.length === 0) return null
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-1.5 text-xs font-semibold tracking-wide text-slate-500 uppercase">
        <FlaskConical className="h-3.5 w-3.5" /> {t('reports.title')} ({reports.length})
      </div>
      <div className="flex flex-wrap gap-2">
        {reports.map((r, i) => (
          <ReportTile key={r.id} r={r} onOpen={() => setOpen(i)} />
        ))}
      </div>
      {open !== null && (
        <ReportViewer reports={reports} index={open} onIndex={setOpen} onClose={() => setOpen(null)} showValues={showValues} />
      )}
    </div>
  )
}

function ReportTile({ r, onOpen }: { r: Report; onOpen: () => void }) {
  const { t } = useTranslation()
  const thumb = useReportFile(r, isImage(r))
  const flagged = r.results.filter((x) => x.status && x.status !== 'normal')
  const critical = flagged.some((x) => x.status === 'critical')
  return (
    <button
      type="button"
      onClick={onOpen}
      className="group flex w-56 items-center gap-2.5 rounded-xl border border-slate-200 bg-white p-1.5 pr-3 text-left hover:border-brand/40 hover:shadow-md"
      title={t('reports.open')}
    >
      <span className="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-slate-100 text-slate-400">
        {isImage(r) ? (
          thumb.data ? (
            <img src={thumb.data} alt="" className="h-full w-full object-cover transition-transform group-hover:scale-105" />
          ) : (
            <ImageIcon className="h-5 w-5" />
          )
        ) : (
          <FileText className="h-6 w-6 text-red-500" />
        )}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-sm font-medium text-slate-800">{title(r, t('reports.untitled'))}</span>
        <span className="block truncate text-xs text-slate-500">
          {r.report_date ? fmtDate(r.report_date) : t(`reports.state.${r.status}`)}
        </span>
        {flagged.length > 0 && (
          <span className={`block text-xs font-medium ${critical ? 'text-red-600' : 'text-orange-600'}`}>
            {critical ? '⚠ ' : ''}
            {flagged.length} {t('reports.flagged')}
          </span>
        )}
      </span>
    </button>
  )
}
