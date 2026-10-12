import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { AnalysisView, Suggestion } from '../../api/types'
import { useSuggestionAction } from './api'

/** What "Add selected" puts into the prescription's sections. */
export interface Insert {
  diagnosis: string[]
  investigations: string[]
  advice: string[]
  next_plan: string[]
}

/** One tick-box line; advice may come in English and Bangla. */
interface Item {
  key: string
  suggestion: Suggestion
  text: string
  lang?: 'en' | 'bn'
}

const SECTIONS: { section: Suggestion['section']; icon: string; key: string }[] = [
  { section: 'diagnosis', icon: '🩺', key: 'ai.possibleIssues' },
  { section: 'investigation', icon: '🔬', key: 'ai.investigations' },
  { section: 'advice', icon: '💬', key: 'ai.advice' },
  { section: 'next_plan', icon: '📅', key: 'ai.nextPlan' },
  { section: 'red_flag', icon: '🚩', key: 'ai.redFlags' },
]

function itemsOf(s: Suggestion): Item[] {
  if (s.section === 'advice') {
    const out: Item[] = [{ key: `${s.id}-en`, suggestion: s, text: s.text_en, lang: 'en' }]
    if (s.text_bn) out.push({ key: `${s.id}-bn`, suggestion: s, text: s.text_bn, lang: 'bn' })
    return out
  }
  return [{ key: String(s.id), suggestion: s, text: s.edited_text ?? s.text_en }]
}

/**
 * AI Findings (§6.2): summary, findings with their source values, conflicts, and
 * suggestions to tick, edit (✎) or reject (✕) before adding them (FR-33, FR-34, FR-37).
 * Nothing enters the prescription without the doctor's tick.
 */
export function FindingsPanel({ analysis, onInsert }: { analysis: AnalysisView; onInsert: (i: Insert) => void }) {
  const { t } = useTranslation()
  const act = useSuggestionAction()
  const a = analysis.content
  const open = analysis.suggestions.filter((s) => s.action === 'pending')
  const all = open.flatMap(itemsOf)
  const [picked, setPicked] = useState<Set<string>>(
    () => new Set(all.filter((i) => i.suggestion.section !== 'red_flag' && i.suggestion.likelihood !== 'Low').map((i) => i.key)),
  )
  const [texts, setTexts] = useState<Record<string, string>>({})
  const [editingKey, setEditingKey] = useState<string | null>(null)
  const [done, setDone] = useState<Set<number>>(new Set())
  const [rejecting, setRejecting] = useState<{ id: number; reason: string } | null>(null)

  const textOf = (i: Item) => texts[i.key] ?? i.text
  const toggle = (key: string) => {
    const next = new Set(picked)
    if (next.has(key)) next.delete(key)
    else next.add(key)
    setPicked(next)
  }

  const reject = async () => {
    if (!rejecting) return
    await act.mutateAsync({ id: rejecting.id, action: 'rejected', reason: rejecting.reason.trim() || undefined })
    setDone(new Set(done).add(rejecting.id))
    setRejecting(null)
  }

  const addSelected = async () => {
    const chosen = all.filter((i) => picked.has(i.key) && !done.has(i.suggestion.id))
    const out: Insert = { diagnosis: [], investigations: [], advice: [], next_plan: [] }
    for (const i of chosen) {
      const text = textOf(i).trim()
      if (!text) continue
      switch (i.suggestion.section) {
        case 'diagnosis':
          out.diagnosis.push(text)
          break
        case 'investigation':
          out.investigations.push(text)
          break
        case 'advice':
          out.advice.push(text)
          break
        case 'next_plan':
          out.next_plan.push(text)
          break
        case 'red_flag':
          out.advice.push(t('ai.returnIf', { what: text }))
          break
      }
    }
    onInsert(out)
    // One decision per suggestion, even when both languages of an advice were added.
    const ids = new Map<number, { edited?: string }>()
    for (const i of chosen) {
      const edited = texts[i.key] !== undefined && texts[i.key] !== i.text ? texts[i.key] : undefined
      ids.set(i.suggestion.id, { edited: ids.get(i.suggestion.id)?.edited ?? edited })
    }
    await Promise.all(
      [...ids].map(([id, { edited }]) =>
        act.mutateAsync({ id, action: edited ? 'edited' : 'accepted', edited_text: edited }),
      ),
    )
    setDone(new Set([...done, ...ids.keys()]))
  }

  const remaining = all.filter((i) => !done.has(i.suggestion.id))

  return (
    <div className="space-y-3 text-sm">
      <section>
        <h3 className="font-semibold">📝 {t('ai.summary')}</h3>
        <p>{a.summary}</p>
        {a.report_summaries?.length > 0 && (
          <ul className="mt-1 space-y-0.5">
            {a.report_summaries.map((r) => (
              <li key={r.report}>
                <b>{r.report}:</b> {r.summary}
              </li>
            ))}
          </ul>
        )}
      </section>

      {a.findings?.length > 0 && (
        <section>
          <h3 className="font-semibold">🔎 {t('ai.findings')}</h3>
          <ul className="ml-4 list-disc space-y-0.5">
            {a.findings.map((f, k) => (
              <li key={k}>
                {f.text}
                {f.source.length > 0 && <span className="text-xs text-slate-500"> — {f.source.join(', ')}</span>}
              </li>
            ))}
          </ul>
        </section>
      )}

      {a.trend_comments?.length > 0 && (
        <section>
          <h3 className="font-semibold">📈 {t('ai.trends')}</h3>
          <ul className="ml-4 list-disc">
            {a.trend_comments.map((c) => (
              <li key={c}>{c}</li>
            ))}
          </ul>
        </section>
      )}

      {a.conflicts?.length > 0 && (
        <section className="rounded bg-amber-50 p-2 text-amber-900">
          <h3 className="font-semibold">⚠ {t('ai.conflicts')}</h3>
          <ul className="ml-4 list-disc">
            {a.conflicts.map((c) => (
              <li key={c}>{c}</li>
            ))}
          </ul>
        </section>
      )}

      {SECTIONS.map(({ section, icon, key }) => {
        const items = remaining.filter((i) => i.suggestion.section === section)
        if (items.length === 0) return null
        return (
          <section key={section}>
            <h3 className="font-semibold">
              {icon} {t(key)}
            </h3>
            <ul className="space-y-1">
              {items.map((i) => (
                <li key={i.key} className="flex flex-wrap items-start gap-2">
                  <input
                    type="checkbox"
                    checked={picked.has(i.key)}
                    onChange={() => toggle(i.key)}
                    className="mt-1"
                    aria-label={textOf(i)}
                  />
                  <div className="min-w-0 flex-1">
                    {editingKey === i.key ? (
                      <input
                        value={textOf(i)}
                        onChange={(e) => setTexts({ ...texts, [i.key]: e.target.value })}
                        onBlur={() => setEditingKey(null)}
                        autoFocus
                        className="w-full rounded border px-1.5 py-0.5"
                      />
                    ) : (
                      <span lang={i.lang === 'bn' ? 'bn' : undefined}>{textOf(i)}</span>
                    )}
                    {i.suggestion.likelihood && (
                      <span className="ml-1 rounded bg-slate-100 px-1 text-xs">{t(`ai.likelihood.${i.suggestion.likelihood}`, i.suggestion.likelihood)}</span>
                    )}
                    {i.suggestion.source_refs.filter(Boolean).length > 0 && (
                      <div className="text-xs text-slate-500">{i.suggestion.source_refs.join('; ')}</div>
                    )}
                  </div>
                  <button type="button" onClick={() => setEditingKey(i.key)} className="px-1 text-slate-500" aria-label={t('ai.edit')}>
                    ✎
                  </button>
                  <button
                    type="button"
                    onClick={() => setRejecting({ id: i.suggestion.id, reason: '' })}
                    className="px-1 text-red-600"
                    aria-label={t('ai.reject')}
                  >
                    ✕
                  </button>
                  {rejecting?.id === i.suggestion.id && (
                    <div className="flex w-full basis-full gap-1 pl-6">
                      <input
                        value={rejecting.reason}
                        onChange={(e) => setRejecting({ ...rejecting, reason: e.target.value })}
                        onKeyDown={(e) => e.key === 'Enter' && reject()}
                        placeholder={t('ai.rejectReason')}
                        maxLength={255}
                        autoFocus
                        className="min-w-0 flex-1 rounded border px-1.5 py-0.5 text-xs"
                      />
                      <button type="button" onClick={reject} disabled={act.isPending} className="rounded border border-red-600 px-2 text-xs text-red-700">
                        {t('ai.reject')}
                      </button>
                      <button type="button" onClick={() => setRejecting(null)} className="px-1 text-xs">
                        {t('patients.cancel')}
                      </button>
                    </div>
                  )}
                </li>
              ))}
            </ul>
          </section>
        )
      })}

      {remaining.length > 0 ? (
        <button
          onClick={addSelected}
          disabled={act.isPending || remaining.every((i) => !picked.has(i.key))}
          className="w-full rounded bg-brand px-3 py-2 text-white disabled:opacity-50"
        >
          {t('ai.addSelected')}
        </button>
      ) : (
        open.length > 0 && <p className="text-green-700">✔ {t('ai.allReviewed')}</p>
      )}
      {act.isError && <p className="text-red-600">{act.error.message}</p>}

      <p className="border-t pt-2 text-xs text-slate-500">
        ⚠ {t('ai.disclaimer')} · {analysis.claude_model} · {t('ai.prompt')} {analysis.prompt_version} ·{' '}
        {new Date(analysis.created_at).toLocaleString()}
      </p>
    </div>
  )
}
