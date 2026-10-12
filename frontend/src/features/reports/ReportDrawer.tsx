import { useRef, useState, type DragEvent } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { fileUrl, streamAi } from '../../api/stream'
import type { AnalysisView, MedicineLine, Report } from '../../api/types'
import { useDeleteReport, useExtractReport, useLatestAnalysis, useUploadReport, useVisitReports } from './api'
import { ExtractedValuesTable } from './ExtractedValuesTable'
import { FindingsPanel, type Insert } from './FindingsPanel'
import { compressImage, shortLine } from './lab'

interface Upload {
  key: number
  name: string
  progress: number
  error?: string
  duplicate?: boolean
}

/** What is on the ℞ screen now, sent with the analysis for correlation (FR-22). */
export interface ScreenRx {
  complaints: string[]
  examination: string[]
  diagnosis: string[]
  investigations: string[]
  medicines: MedicineLine[]
}

/**
 * 📄 Reports for this visit (AI spec §4, §6): upload → AI reads the values → doctor
 * checks them → AI findings → tick suggestions into the prescription.
 */
export function ReportDrawer({
  visitId,
  rx,
  aiAvailable,
  onInsert,
  onClose,
}: {
  visitId: number
  rx: ScreenRx
  aiAvailable: boolean
  onInsert: (i: Insert) => void
  onClose: () => void
}) {
  const { t } = useTranslation()
  const qc = useQueryClient()
  const reports = useVisitReports(visitId)
  const upload = useUploadReport(visitId)
  const extract = useExtractReport()
  const del = useDeleteReport()
  const latest = useLatestAnalysis(visitId)
  const [uploads, setUploads] = useState<Upload[]>([])
  const [dragging, setDragging] = useState(false)
  const [analysing, setAnalysing] = useState<number | null>(null)
  const [analysisError, setAnalysisError] = useState<string | null>(null)
  const [manual, setManual] = useState<Set<number>>(new Set())
  const fileInput = useRef<HTMLInputElement>(null)
  const cameraInput = useRef<HTMLInputElement>(null)
  const counter = useRef(0)

  const send = async (files: FileList | File[]) => {
    for (const file of Array.from(files)) {
      const key = ++counter.current
      setUploads((u) => [...u, { key, name: file.name, progress: 0 }])
      const patch = (p: Partial<Upload>) => setUploads((u) => u.map((x) => (x.key === key ? { ...x, ...p } : x)))
      try {
        const body = await compressImage(file)
        const res = await upload.mutateAsync({ file: body, name: file.name, onProgress: (progress) => patch({ progress }) })
        patch({ progress: 1, duplicate: res.duplicate })
        if (!res.duplicate) setTimeout(() => setUploads((u) => u.filter((x) => x.key !== key)), 1500)
      } catch (e) {
        patch({ error: (e as Error).message })
      }
    }
  }

  const drop = (e: DragEvent) => {
    e.preventDefault()
    setDragging(false)
    if (e.dataTransfer.files.length) send(e.dataTransfer.files)
  }

  const analyse = async () => {
    setAnalysing(0)
    setAnalysisError(null)
    try {
      const result = await streamAi<AnalysisView>(`/visits/${visitId}/ai-analysis`, rx, setAnalysing)
      qc.setQueryData(['analysis', visitId], result)
    } catch (e) {
      setAnalysisError((e as Error).message)
    } finally {
      setAnalysing(null)
    }
  }

  const openFile = async (r: Report) => {
    const url = await fileUrl(`/reports/${r.id}/file`)
    window.open(url, '_blank', 'noopener')
  }

  const list = reports.data ?? []
  const readable = list.filter((r) => r.status === 'extracted' || r.status === 'verified')
  const shortLines = readable.map(shortLine).filter((l): l is string => !!l)

  return (
    <div className="fixed inset-0 z-30 print:hidden" role="dialog" aria-label={t('reports.title')}>
      <div className="absolute inset-0 bg-black/30" onClick={onClose} />
      <aside className="absolute inset-y-0 right-0 flex w-full max-w-2xl flex-col bg-white shadow-xl">
        <div className="flex items-center border-b px-4 py-3">
          <h2 className="font-semibold">📄 {t('reports.title')}</h2>
          <button onClick={onClose} className="ml-auto px-2 text-lg" aria-label={t('common.close')}>
            ✕
          </button>
        </div>
        <div className="flex-1 space-y-4 overflow-y-auto p-4">
          <div
            onDragOver={(e) => {
              e.preventDefault()
              setDragging(true)
            }}
            onDragLeave={() => setDragging(false)}
            onDrop={drop}
            className={`rounded-lg border-2 border-dashed p-4 text-center text-sm ${dragging ? 'border-brand bg-blue-50' : 'border-slate-300'}`}
          >
            <p className="text-slate-500">{t('reports.dropHere')}</p>
            <div className="mt-2 flex flex-wrap justify-center gap-2">
              <button onClick={() => fileInput.current?.click()} className="rounded bg-brand px-3 py-1.5 text-white">
                📎 {t('reports.chooseFiles')}
              </button>
              <button onClick={() => cameraInput.current?.click()} className="rounded border px-3 py-1.5">
                📷 {t('reports.camera')}
              </button>
            </div>
            <input
              ref={fileInput}
              type="file"
              multiple
              accept="application/pdf,image/jpeg,image/png,image/webp"
              className="hidden"
              onChange={(e) => {
                if (e.target.files) send(e.target.files)
                e.target.value = ''
              }}
            />
            <input
              ref={cameraInput}
              type="file"
              accept="image/*"
              capture="environment"
              className="hidden"
              onChange={(e) => {
                if (e.target.files) send(e.target.files)
                e.target.value = ''
              }}
            />
            <p className="mt-2 text-xs text-slate-400">{t('reports.fileTypes')}</p>
          </div>

          {uploads.map((u) => (
            <div key={u.key} className="text-sm">
              <div className="flex justify-between">
                <span className="truncate">{u.name}</span>
                <span className={u.error ? 'text-red-600' : 'text-slate-500'}>
                  {u.error
                    ? t('reports.failed')
                    : u.duplicate
                      ? t('reports.duplicate')
                      : u.progress < 1
                        ? `${Math.round(u.progress * 100)}%`
                        : `🤖 ${t('reports.reading')}`}
                </span>
              </div>
              <div className="mt-1 h-1.5 rounded bg-slate-200">
                <div className={`h-1.5 rounded ${u.error ? 'bg-red-500' : 'bg-brand'}`} style={{ width: `${u.progress * 100}%` }} />
              </div>
              {u.error && <p className="text-xs text-red-600">{u.error}</p>}
            </div>
          ))}

          {list.map((r) => (
            <section key={r.id} className="space-y-2 rounded border p-3">
              <div className="flex flex-wrap items-center gap-2 text-sm">
                <b>{r.report_type ?? r.original_name ?? t('reports.untitled')}</b>
                <span className="rounded bg-slate-100 px-1.5 text-xs">{t(`reports.state.${r.status}`)}</span>
                <span className="ml-auto flex gap-1">
                  <button onClick={() => openFile(r)} className="rounded border px-2 py-0.5 text-xs">
                    {t('reports.viewFile')} ↗
                  </button>
                  {aiAvailable && (r.status === 'uploaded' || r.status === 'failed') && (
                    <button
                      onClick={() => extract.mutate(r.id)}
                      disabled={extract.isPending}
                      className="rounded border border-brand px-2 py-0.5 text-xs text-brand disabled:opacity-50"
                    >
                      🤖 {extract.isPending && extract.variables === r.id ? t('reports.reading') : t('reports.read')}
                    </button>
                  )}
                  <button
                    onClick={() => del.mutate(r.id)}
                    disabled={del.isPending}
                    className="px-1 text-xs text-red-600"
                    aria-label={t('reports.delete')}
                    title={t('reports.delete')}
                  >
                    🗑
                  </button>
                </span>
              </div>
              {r.status === 'failed' && <p className="text-sm text-red-600">{r.error ?? t('reports.unclear')}</p>}
              {r.status === 'extracted' || r.status === 'verified' || manual.has(r.id) ? (
                <ExtractedValuesTable report={r} startEditing={manual.has(r.id) || undefined} />
              ) : (
                (r.status === 'uploaded' || r.status === 'failed') && (
                  <button onClick={() => setManual(new Set(manual).add(r.id))} className="text-sm text-brand">
                    ✎ {t('reports.enterManually')}
                  </button>
                )
              )}
            </section>
          ))}
          {extract.isError && <p className="text-sm text-red-600">{extract.error.message}</p>}

          {shortLines.length > 0 && (
            <button
              onClick={() => onInsert({ diagnosis: [], investigations: shortLines, advice: [], next_plan: [] })}
              className="w-full rounded border border-brand px-3 py-1.5 text-sm text-brand"
            >
              ➕ {t('reports.addToInvestigations')}
            </button>
          )}

          {readable.length > 0 && aiAvailable && (
            <section className="space-y-3 rounded-lg bg-panel p-3">
              <div className="flex items-center gap-2">
                <h3 className="font-semibold">🤖 {t('ai.analysis')}</h3>
                <button
                  onClick={analyse}
                  disabled={analysing !== null}
                  className="ml-auto rounded bg-brand px-3 py-1 text-sm text-white disabled:opacity-60"
                >
                  {analysing !== null
                    ? `${t('ai.analysing')}${analysing > 0 ? ` (${analysing})` : ''}`
                    : latest.data
                      ? `↻ ${t('ai.reanalyse')}`
                      : t('ai.analyse')}
                </button>
              </div>
              {analysisError && <p className="text-sm text-red-600">{analysisError}</p>}
              {latest.data && <FindingsPanel key={latest.data.id} analysis={latest.data} onInsert={onInsert} />}
            </section>
          )}
          {!aiAvailable && <p className="text-xs text-slate-500">{t('ai.off')}</p>}
        </div>
      </aside>
    </div>
  )
}
