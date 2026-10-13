import { useCallback, useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ChevronLeft,
  ChevronRight,
  Download,
  ExternalLink,
  FlaskConical,
  Maximize,
  RotateCcw,
  RotateCw,
  X,
  ZoomIn,
  ZoomOut,
} from 'lucide-react'
import type { Report } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { statusStyle } from './lab'
import { isImage, reportTitle, useReportFile } from './reportFile'

const MIN_ZOOM = 0.5
const MAX_ZOOM = 6
const clamp = (z: number) => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z))

/**
 * Full-screen report viewer: the file itself (images can be zoomed, rotated and
 * dragged), step through several reports, and for doctors the values read from it.
 */
export function ReportViewer({
  reports,
  index,
  onIndex,
  onClose,
  showValues,
}: {
  reports: Report[]
  index: number
  onIndex: (i: number) => void
  onClose: () => void
  showValues: boolean
}) {
  const { t } = useTranslation()
  const r = reports[index]
  const file = useReportFile(r)
  const many = reports.length > 1
  const values = showValues && r.results.length > 0

  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
      if (many && e.key === 'ArrowRight') onIndex((index + 1) % reports.length)
      if (many && e.key === 'ArrowLeft') onIndex((index - 1 + reports.length) % reports.length)
    }
    window.addEventListener('keydown', key)
    return () => window.removeEventListener('keydown', key)
  }, [index, many, onClose, onIndex, reports.length])

  const go = (d: number) => onIndex((index + d + reports.length) % reports.length)
  const iconBtn = 'rounded-lg p-2 text-white/80 hover:bg-white/10 hover:text-white'

  return (
    <div
      className="fixed inset-0 z-40 flex flex-col bg-slate-950/95 backdrop-blur-sm print:hidden"
      role="dialog"
      aria-modal="true"
      aria-label={reportTitle(r, t('reports.untitled'))}
    >
      <header className="flex flex-wrap items-center gap-3 px-4 py-3 text-white">
        <FlaskConical className="h-5 w-5 shrink-0 text-sky-300" />
        <div className="min-w-0">
          <div className="truncate font-semibold">{reportTitle(r, t('reports.untitled'))}</div>
          <div className="truncate text-xs text-white/60">
            {[r.report_date && fmtDate(r.report_date), r.lab_name, r.original_name, many && `${index + 1} / ${reports.length}`]
              .filter(Boolean)
              .join(' · ')}
          </div>
        </div>
        <div className="ml-auto flex items-center gap-1">
          {file.data && (
            <>
              <a href={file.data} target="_blank" rel="noopener" className={iconBtn} title={t('reports.newTab')} aria-label={t('reports.newTab')}>
                <ExternalLink className="h-5 w-5" />
              </a>
              <a
                href={file.data}
                download={r.original_name ?? `report-${r.id}`}
                className={iconBtn}
                title={t('reports.download')}
                aria-label={t('reports.download')}
              >
                <Download className="h-5 w-5" />
              </a>
            </>
          )}
          <button onClick={onClose} className={iconBtn} aria-label={t('common.close')} title={t('common.close')}>
            <X className="h-5 w-5" />
          </button>
        </div>
      </header>

      <div className={`grid min-h-0 flex-1 gap-4 px-4 pb-4 ${values ? 'lg:grid-cols-[1fr_380px]' : ''}`}>
        <div className="relative min-h-0 overflow-hidden rounded-xl bg-black/40">
          {!file.data ? (
            <p className="flex h-full items-center justify-center text-white/70">
              {file.isError ? file.error.message : t('common.loading')}
            </p>
          ) : isImage(r) ? (
            // A new key per report starts each image at the default view.
            <ImageStage key={r.id} src={file.data} alt={reportTitle(r, '')} />
          ) : (
            <iframe src={file.data} title={reportTitle(r, '')} className="h-full w-full rounded-xl bg-white" />
          )}
          {many && (
            <>
              <button
                onClick={() => go(-1)}
                className="absolute top-1/2 left-3 z-10 -translate-y-1/2 rounded-full bg-white/15 p-2 text-white hover:bg-white/25"
                aria-label={t('common.prev')}
              >
                <ChevronLeft className="h-6 w-6" />
              </button>
              <button
                onClick={() => go(1)}
                className="absolute top-1/2 right-3 z-10 -translate-y-1/2 rounded-full bg-white/15 p-2 text-white hover:bg-white/25"
                aria-label={t('common.next')}
              >
                <ChevronRight className="h-6 w-6" />
              </button>
            </>
          )}
        </div>

        {values && (
          <aside className="min-h-0 overflow-y-auto rounded-xl bg-white p-4 text-sm">
            <h3 className="mb-2 font-semibold text-slate-900">{t('reports.values')}</h3>
            <table className="w-full text-left">
              <tbody>
                {r.results.map((x) => {
                  const s = x.status ? statusStyle[x.status] : null
                  return (
                    <tr key={x.id} className="border-b last:border-0">
                      <td className="py-1.5 pr-2">{x.test_name}</td>
                      <td className={`py-1.5 pr-2 whitespace-nowrap ${s?.cls ?? ''}`}>
                        {x.value_text} <span className="text-xs text-slate-500">{x.unit}</span>
                      </td>
                      <td className={`py-1.5 text-right whitespace-nowrap ${s?.cls ?? 'text-slate-400'}`}>{s ? s.icon : ''}</td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
            {r.impression && <p className="mt-3 whitespace-pre-line text-slate-700">{r.impression}</p>}
          </aside>
        )}
      </div>

      {many && (
        <div className="flex justify-center gap-2 overflow-x-auto px-4 pb-4">
          {reports.map((x, i) => (
            <button
              key={x.id}
              onClick={() => onIndex(i)}
              className={`h-1.5 w-8 shrink-0 rounded-full ${i === index ? 'bg-white' : 'bg-white/30 hover:bg-white/50'}`}
              aria-label={`${i + 1}`}
            />
          ))}
        </div>
      )}
    </div>
  )
}

/**
 * An image that can be zoomed (buttons, wheel, + / − keys, double-click), rotated in
 * 90° steps (R / Shift+R) and dragged when zoomed. Rotated images are refitted so the
 * whole page stays visible.
 */
function ImageStage({ src, alt }: { src: string; alt: string }) {
  const { t } = useTranslation()
  const stage = useRef<HTMLDivElement>(null)
  const img = useRef<HTMLImageElement>(null)
  const [zoom, setZoom] = useState(1)
  const [rotation, setRotation] = useState(0)
  const [pan, setPan] = useState({ x: 0, y: 0 })
  const [fit, setFit] = useState(1)
  const drag = useRef<{ x: number; y: number; px: number; py: number } | null>(null)
  const [dragging, setDragging] = useState(false)

  // Sideways (90° / 270°) the image's width and height swap; shrink it to still fit.
  const refit = useCallback(() => {
    const s = stage.current
    const i = img.current
    if (!s || !i || !i.offsetWidth) return
    const sideways = rotation % 180 !== 0
    setFit(sideways ? Math.min(1, s.clientWidth / i.offsetHeight, s.clientHeight / i.offsetWidth) : 1)
  }, [rotation])

  useEffect(() => {
    refit()
    window.addEventListener('resize', refit)
    return () => window.removeEventListener('resize', refit)
  }, [refit])

  const zoomBy = useCallback((f: number) => {
    setZoom((z) => {
      const next = clamp(z * f)
      if (next <= 1) setPan({ x: 0, y: 0 })
      return next
    })
  }, [])
  const rotate = useCallback((d: number) => {
    setRotation((r) => r + d)
    setPan({ x: 0, y: 0 })
  }, [])
  const reset = useCallback(() => {
    setZoom(1)
    setRotation(0)
    setPan({ x: 0, y: 0 })
  }, [])

  // Wheel zoom needs a non-passive listener so the page does not scroll.
  useEffect(() => {
    const s = stage.current
    if (!s) return
    const wheel = (e: WheelEvent) => {
      e.preventDefault()
      zoomBy(e.deltaY < 0 ? 1.15 : 1 / 1.15)
    }
    s.addEventListener('wheel', wheel, { passive: false })
    return () => s.removeEventListener('wheel', wheel)
  }, [zoomBy])

  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return
      if (e.key === '+' || e.key === '=') zoomBy(1.25)
      else if (e.key === '-' || e.key === '_') zoomBy(1 / 1.25)
      else if (e.key === '0') reset()
      else if (e.key.toLowerCase() === 'r') rotate(e.shiftKey ? -90 : 90)
    }
    window.addEventListener('keydown', key)
    return () => window.removeEventListener('keydown', key)
  }, [zoomBy, rotate, reset])

  const down = (e: ReactPointerEvent) => {
    if (zoom <= 1) return
    e.currentTarget.setPointerCapture(e.pointerId)
    drag.current = { x: e.clientX, y: e.clientY, px: pan.x, py: pan.y }
    setDragging(true)
  }
  const move = (e: ReactPointerEvent) => {
    const d = drag.current
    if (d) setPan({ x: d.px + e.clientX - d.x, y: d.py + e.clientY - d.y })
  }
  const up = () => {
    drag.current = null
    setDragging(false)
  }

  const tool = 'rounded-lg p-2 text-white/80 hover:bg-white/15 hover:text-white disabled:opacity-30'
  return (
    <div className="relative h-full w-full">
      <div
        ref={stage}
        onPointerDown={down}
        onPointerMove={move}
        onPointerUp={up}
        onPointerCancel={up}
        onDoubleClick={() => (zoom > 1 ? reset() : setZoom(2.5))}
        className={`flex h-full w-full touch-none items-center justify-center overflow-hidden select-none ${
          zoom > 1 ? (dragging ? 'cursor-grabbing' : 'cursor-grab') : 'cursor-zoom-in'
        }`}
      >
        <img
          ref={img}
          src={src}
          alt={alt}
          draggable={false}
          onLoad={refit}
          className={`max-h-full max-w-full object-contain shadow-2xl ${dragging ? '' : 'transition-transform duration-150 ease-out'}`}
          style={{ transform: `translate(${pan.x}px, ${pan.y}px) rotate(${rotation}deg) scale(${zoom * fit})` }}
        />
      </div>

      {/* Tools */}
      <div className="absolute bottom-3 left-1/2 z-10 flex -translate-x-1/2 items-center gap-0.5 rounded-xl bg-slate-900/80 p-1 shadow-lg ring-1 ring-white/10 backdrop-blur">
        <button onClick={() => zoomBy(1 / 1.25)} disabled={zoom <= MIN_ZOOM} className={tool} title={`${t('viewer.zoomOut')} (−)`} aria-label={t('viewer.zoomOut')}>
          <ZoomOut className="h-5 w-5" />
        </button>
        <button onClick={reset} className="min-w-14 rounded-lg px-2 py-2 text-sm font-medium text-white tabular-nums hover:bg-white/15" title={`${t('viewer.reset')} (0)`}>
          {Math.round(zoom * 100)}%
        </button>
        <button onClick={() => zoomBy(1.25)} disabled={zoom >= MAX_ZOOM} className={tool} title={`${t('viewer.zoomIn')} (+)`} aria-label={t('viewer.zoomIn')}>
          <ZoomIn className="h-5 w-5" />
        </button>
        <span className="mx-1 h-5 w-px bg-white/20" />
        <button onClick={() => rotate(-90)} className={tool} title={`${t('viewer.rotateLeft')} (Shift+R)`} aria-label={t('viewer.rotateLeft')}>
          <RotateCcw className="h-5 w-5" />
        </button>
        <button onClick={() => rotate(90)} className={tool} title={`${t('viewer.rotateRight')} (R)`} aria-label={t('viewer.rotateRight')}>
          <RotateCw className="h-5 w-5" />
        </button>
        <span className="mx-1 h-5 w-px bg-white/20" />
        <button onClick={reset} className={tool} title={`${t('viewer.reset')} (0)`} aria-label={t('viewer.reset')}>
          <Maximize className="h-5 w-5" />
        </button>
      </div>
    </div>
  )
}
