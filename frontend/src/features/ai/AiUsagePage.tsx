import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { format, startOfMonth } from 'date-fns'
import { Activity, ArrowDownToLine, ArrowUpFromLine, DatabaseZap } from 'lucide-react'
import { Card } from '../../components/Card'
import { StatCard } from '../../components/StatCard'
import { fmtDate } from '../../utils/date'
import { useAiUsage } from './api'

const iso = (d: Date) => format(d, 'yyyy-MM-dd')
const n = (x: number) => x.toLocaleString('en-IN')

/**
 * AI usage (AI spec §8.9, roadmap 3.9): calls and tokens per day, task and model.
 * Prices change over time, so cost is read off the current price list from these counts.
 */
export function AiUsagePage() {
  const { t } = useTranslation()
  const [range] = useState(() => ({ from: iso(startOfMonth(new Date())), to: iso(new Date()) }))
  const [from, setFrom] = useState(range.from)
  const [to, setTo] = useState(range.to)
  const { data, isError, error } = useAiUsage(from, to)
  const rows = data?.rows ?? []
  const sum = (k: 'calls' | 'failed' | 'input_tokens' | 'output_tokens' | 'cache_read_tokens') =>
    rows.reduce((a, r) => a + r[k], 0)

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-2xl font-bold tracking-tight text-slate-900">🤖 {t('ai.usage')}</h1>
        <div className="ml-auto flex items-center gap-2 text-sm">
          <input type="date" value={from} max={to} onChange={(e) => e.target.value && setFrom(e.target.value)} className="rounded-md border px-2 py-1" aria-label={t('settings.from')} />
          –
          <input type="date" value={to} min={from} onChange={(e) => e.target.value && setTo(e.target.value)} className="rounded-md border px-2 py-1" aria-label={t('settings.to')} />
        </div>
      </div>
      {isError && <p className="text-red-600">{error.message}</p>}
      <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        <StatCard icon={Activity} tone="blue" label={t('ai.calls')} value={data ? n(sum('calls')) : '—'} />
        <StatCard icon={ArrowDownToLine} tone="violet" label={t('ai.inputTokens')} value={data ? n(sum('input_tokens')) : '—'} />
        <StatCard icon={ArrowUpFromLine} tone="amber" label={t('ai.outputTokens')} value={data ? n(sum('output_tokens')) : '—'} />
        <StatCard icon={DatabaseZap} tone="green" label={t('ai.cachedTokens')} value={data ? n(sum('cache_read_tokens')) : '—'} />
      </div>
      <Card>
        {rows.length === 0 ? (
          <p className="text-slate-500">{data ? t('ai.noUsage') : t('common.loading')}</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead className="border-b text-slate-500">
                <tr>
                  <th className="py-2 pr-3">{t('billing.date')}</th>
                  <th className="py-2 pr-3">{t('ai.task')}</th>
                  <th className="py-2 pr-3">{t('ai.model')}</th>
                  <th className="py-2 pr-3 text-right">{t('ai.calls')}</th>
                  <th className="py-2 pr-3 text-right">{t('ai.inputTokens')}</th>
                  <th className="py-2 pr-3 text-right">{t('ai.outputTokens')}</th>
                  <th className="py-2 pr-3 text-right">{t('ai.cachedTokens')}</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr key={`${r.day}-${r.analysis_type}-${r.claude_model}`} className="border-b last:border-0">
                    <td className="py-2 pr-3 whitespace-nowrap">{fmtDate(r.day)}</td>
                    <td className="py-2 pr-3">{t(`ai.tasks.${r.analysis_type}`, r.analysis_type)}</td>
                    <td className="py-2 pr-3 font-mono text-xs">{r.claude_model}</td>
                    <td className="py-2 pr-3 text-right">
                      {n(r.calls)}
                      {r.failed > 0 && <span className="text-red-600"> ({r.failed} ✕)</span>}
                    </td>
                    <td className="py-2 pr-3 text-right">{n(r.input_tokens)}</td>
                    <td className="py-2 pr-3 text-right">{n(r.output_tokens)}</td>
                    <td className="py-2 pr-3 text-right">{n(r.cache_read_tokens)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>
      <p className="text-xs text-slate-500">{t('ai.costNote')}</p>
    </div>
  )
}
