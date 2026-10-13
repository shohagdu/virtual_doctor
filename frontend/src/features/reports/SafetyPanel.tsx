import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { MedicineLine, SafetyWarning } from '../../api/types'
import { useSafetyCheck, useSuggestionAction } from './api'

const severityClass: Record<string, string> = {
  high: 'border-red-400 bg-red-50 text-red-800',
  moderate: 'border-orange-300 bg-orange-50 text-orange-900',
  low: 'border-slate-300 bg-slate-50 text-slate-700',
}

/**
 * 💊 Prescription check under the ℞ panel (FR-29–FR-32): built-in rules plus AI when
 * available. Advisory only; a warning is dismissed with a reason.
 */
export function SafetyPanel({ visitId, medicines }: { visitId: number; medicines: MedicineLine[] }) {
  const { t } = useTranslation()
  const check = useSafetyCheck(visitId)
  const act = useSuggestionAction()
  const [dismissing, setDismissing] = useState<{ id: number; reason: string } | null>(null)
  const [dismissed, setDismissed] = useState<Set<number>>(new Set())
  const lines = medicines.filter((m) => m.brand.trim())

  const dismiss = async () => {
    if (!dismissing?.reason.trim()) return
    await act.mutateAsync({ id: dismissing.id, action: 'rejected', reason: dismissing.reason.trim() })
    setDismissed(new Set(dismissed).add(dismissing.id))
    setDismissing(null)
  }

  const warnings: SafetyWarning[] = (check.data?.warnings ?? []).filter((w) => !dismissed.has(w.id))

  return (
    <div className="space-y-2 text-sm">
      <button
        type="button"
        onClick={() => {
          setDismissed(new Set())
          check.mutate(lines)
        }}
        disabled={check.isPending || lines.length === 0}
        className="rounded-md border px-2 py-1 text-xs disabled:opacity-50"
      >
        💊 {check.isPending ? t('ai.checking') : t('ai.checkRx')}
      </button>
      {check.isError && <p className="text-red-600">{check.error.message}</p>}
      {check.data && warnings.length === 0 && (
        <p className="text-green-700">
          ✔ {t('ai.noWarnings')}
          {!check.data.ai_used && <span className="text-slate-500"> ({t('ai.rulesOnly')})</span>}
        </p>
      )}
      {warnings.map((w) => (
        <div key={w.id} className={`rounded-md border p-2 ${severityClass[w.severity] ?? severityClass.low}`}>
          <div className="flex gap-2">
            <span className="font-semibold">{w.medicines.join(' + ')}</span>
            <span className="text-xs uppercase">{t(`ai.severity.${w.severity}`, w.severity)}</span>
            <span className="ml-auto text-xs opacity-70">{w.origin === 'rules' ? t('ai.rule') : 'AI'}</span>
          </div>
          <p>{w.message}</p>
          {w.source.length > 0 && <p className="text-xs opacity-80">{w.source.join('; ')}</p>}
          {dismissing?.id === w.id ? (
            <div className="mt-1 flex gap-1">
              <input
                value={dismissing.reason}
                onChange={(e) => setDismissing({ ...dismissing, reason: e.target.value })}
                onKeyDown={(e) => e.key === 'Enter' && dismiss()}
                placeholder={t('ai.dismissReason')}
                maxLength={255}
                autoFocus
                className="min-w-0 flex-1 rounded-md border bg-white px-1.5 py-0.5 text-xs"
              />
              <button type="button" onClick={dismiss} disabled={!dismissing.reason.trim() || act.isPending} className="rounded-md border bg-white px-2 text-xs disabled:opacity-50">
                {t('ai.dismiss')}
              </button>
            </div>
          ) : (
            <button type="button" onClick={() => setDismissing({ id: w.id, reason: '' })} className="mt-1 text-xs underline">
              {t('ai.dismiss')}
            </button>
          )}
        </div>
      ))}
      {check.data && <p className="text-xs text-slate-500">{t('ai.advisory')}</p>}
    </div>
  )
}
