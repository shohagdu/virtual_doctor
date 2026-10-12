import { useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { streamAi } from '../../api/stream'
import type { PatientSummary, SavedSummary } from '../../api/types'
import { useLatestSummary } from './api'

const SECTIONS: { key: keyof Omit<PatientSummary, 'one_line'>; label: string }[] = [
  { key: 'alerts', label: 'ai.s.alerts' },
  { key: 'active_problems', label: 'ai.s.activeProblems' },
  { key: 'recurring_issues', label: 'ai.s.recurring' },
  { key: 'lab_trends', label: 'ai.s.labTrends' },
  { key: 'medicines_history', label: 'ai.s.medicines' },
  { key: 'points_to_check', label: 'ai.s.pointsToCheck' },
]

/**
 * 🤖 AI patient summary (PM spec §7.1): the last saved summary, and a button to
 * generate a new one. Shows date, model and prompt version (§7.4).
 */
export function AiSummary({ patientId, available }: { patientId: string; available: boolean }) {
  const { t } = useTranslation()
  const qc = useQueryClient()
  const { data, isLoading } = useLatestSummary(patientId)
  const [progress, setProgress] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)

  const generate = async () => {
    setProgress(0)
    setError(null)
    try {
      const s = await streamAi<SavedSummary>(`/patients/${patientId}/ai-summary`, undefined, setProgress)
      qc.setQueryData(['ai-summary', patientId], s)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setProgress(null)
    }
  }

  return (
    <div className="space-y-3 text-sm">
      <div className="flex flex-wrap items-center gap-2">
        {data && <span className="text-xs text-slate-500">{t('ai.generatedOn', { date: new Date(data.created_at).toLocaleString() })}</span>}
        {available && (
          <button
            onClick={generate}
            disabled={progress !== null}
            className="ml-auto rounded bg-brand px-3 py-1.5 text-white disabled:opacity-60"
          >
            🤖{' '}
            {progress !== null
              ? `${t('ai.summarising')}${progress > 0 ? ` (${progress})` : ''}`
              : data
                ? t('ai.refreshSummary')
                : t('ai.summarise')}
          </button>
        )}
      </div>
      {error && <p className="text-red-600">{error}</p>}
      {isLoading ? (
        <p className="text-slate-500">{t('common.loading')}</p>
      ) : !data ? (
        <p className="text-slate-500">{available ? t('ai.noSummary') : t('ai.off')}</p>
      ) : (
        <>
          <p className="rounded bg-infobar px-3 py-2 font-medium">{data.content.one_line}</p>
          <div className="grid gap-3 md:grid-cols-2">
            {SECTIONS.map(({ key, label }) => {
              const items = data.content[key]
              if (!items?.length) return null
              return (
                <section key={key} className={key === 'alerts' ? 'rounded bg-red-50 p-2 text-red-800' : ''}>
                  <h3 className="font-semibold">{t(label)}</h3>
                  <ul className="ml-4 list-disc">
                    {items.map((x) => (
                      <li key={x}>{x}</li>
                    ))}
                  </ul>
                </section>
              )
            })}
          </div>
          <p className="border-t pt-2 text-xs text-slate-500">
            ⚠ {t('ai.disclaimer')} · {data.claude_model} · {t('ai.prompt')} {data.prompt_version}
          </p>
        </>
      )}
    </div>
  )
}
