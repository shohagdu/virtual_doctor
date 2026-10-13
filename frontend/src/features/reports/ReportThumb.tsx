import { useTranslation } from 'react-i18next'
import { FileText, ImageIcon, Maximize2 } from 'lucide-react'
import type { Report } from '../../api/types'
import { isImage, useReportFile } from './reportFile'

/** A small clickable preview of a report file (image thumbnail, or a PDF icon). */
export function ReportThumb({ r, onClick }: { r: Report; onClick: () => void }) {
  const { t } = useTranslation()
  const file = useReportFile(r, isImage(r))
  return (
    <button
      type="button"
      onClick={onClick}
      className="group relative flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-slate-100 text-slate-400 ring-1 ring-slate-200 hover:ring-brand"
      title={t('reports.open')}
      aria-label={t('reports.open')}
    >
      {isImage(r) ? (
        file.data ? <img src={file.data} alt="" className="h-full w-full object-cover" /> : <ImageIcon className="h-5 w-5" />
      ) : (
        <FileText className="h-6 w-6 text-red-500" />
      )}
      <span className="absolute inset-0 flex items-center justify-center bg-slate-900/40 text-white opacity-0 group-hover:opacity-100">
        <Maximize2 className="h-4 w-4" />
      </span>
    </button>
  )
}
