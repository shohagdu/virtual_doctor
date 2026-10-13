import { useState } from 'react'
import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import { format, startOfMonth, subDays } from 'date-fns'
import { BadgePercent, Banknote, Clock, Receipt } from 'lucide-react'
import { Card } from '../../components/Card'
import { StatCard } from '../../components/StatCard'
import { fmtDate, todayIso } from '../../utils/date'
import { taka } from '../../utils/money'
import { useCollection } from './api'

const iso = (d: Date) => format(d, 'yyyy-MM-dd')

function makePresets() {
  const now = new Date()
  const yesterday = iso(subDays(now, 1))
  return [
    { key: 'billing.today', from: iso(now), to: iso(now) },
    { key: 'billing.yesterday', from: yesterday, to: yesterday },
    { key: 'billing.thisMonth', from: iso(startOfMonth(now)), to: iso(now) },
  ]
}

/** Daily collection report for the doctor / admin (§5.8), by visit date. */
export function CollectionPage() {
  const { t } = useTranslation()
  const [from, setFrom] = useState(todayIso)
  const [to, setTo] = useState(todayIso)
  const { data, isError, error, isFetching } = useCollection(from, to)

  const [presets] = useState(makePresets)
  const tot = data?.totals

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2 print:hidden">
        <h1 className="text-2xl font-bold tracking-tight text-slate-900">{t('billing.collection')}</h1>
        <div className="ml-auto flex flex-wrap items-center gap-2 text-sm">
          {presets.map((p) => (
            <button
              key={p.key}
              onClick={() => {
                setFrom(p.from)
                setTo(p.to)
              }}
              aria-pressed={from === p.from && to === p.to}
              className={`rounded-md px-3 py-1.5 ${from === p.from && to === p.to ? 'bg-brand text-white' : 'border bg-white'}`}
            >
              {t(p.key)}
            </button>
          ))}
          <input
            type="date"
            value={from}
            max={to}
            onChange={(e) => e.target.value && setFrom(e.target.value)}
            className="rounded-md border px-2 py-1"
            aria-label={t('settings.from')}
          />
          –
          <input
            type="date"
            value={to}
            min={from}
            onChange={(e) => e.target.value && setTo(e.target.value)}
            className="rounded-md border px-2 py-1"
            aria-label={t('settings.to')}
          />
          <button onClick={() => window.print()} className="rounded-md border bg-white px-3 py-1.5">
            🖨 {t('billing.print')}
          </button>
        </div>
      </div>
      <p className="hidden print:block">
        <b>{t('billing.collection')}</b>: {fmtDate(from)}
        {to !== from && <> – {fmtDate(to)}</>}
      </p>

      {isError && <p className="text-red-600">{error.message}</p>}

      <div className={`grid grid-cols-2 gap-4 lg:grid-cols-4 ${isFetching ? 'opacity-70' : ''}`}>
        <StatCard icon={Banknote} tone="green" label={t('billing.collected')} value={tot ? taka(tot.paid) : '—'} />
        <StatCard icon={Clock} tone="amber" label={t('billing.due')} value={tot ? taka(tot.due) : '—'} />
        <StatCard icon={BadgePercent} tone="violet" label={t('billing.discount')} value={tot ? taka(tot.discount) : '—'} />
        <StatCard icon={Receipt} tone="blue" label={t('billing.bills')} value={tot?.count ?? '—'} />
      </div>

      {tot && (
        <Card title={t('billing.byMethod')}>
          <table className="w-full max-w-md text-left text-sm">
            <tbody>
              {tot.by_method.map((m) => (
                <tr key={m.method} className="border-b last:border-0">
                  <td className="py-1.5">{t(`billing.methods.${m.method}`)}</td>
                  <td className="py-1.5 text-slate-500">{t('billing.nBills', { count: m.count })}</td>
                  <td className="py-1.5 text-right font-medium">{taka(m.paid)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}

      <Card title={t('billing.bills')}>
        {!data ? (
          <p className="text-slate-500">{t('common.loading')}</p>
        ) : data.rows.length === 0 ? (
          <p className="text-slate-500">{t('billing.none')}</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead className="border-b text-slate-500">
                <tr>
                  <th className="py-2 pr-3">{t('billing.receiptNo')}</th>
                  <th className="py-2 pr-3">{t('billing.date')}</th>
                  <th className="py-2 pr-3">{t('patients.name')}</th>
                  <th className="py-2 pr-3">{t('queue.type')}</th>
                  <th className="py-2 pr-3 text-right">{t('billing.fee')}</th>
                  <th className="py-2 pr-3 text-right">{t('billing.discount')}</th>
                  <th className="py-2 pr-3 text-right">{t('billing.paid')}</th>
                  <th className="py-2 pr-3 text-right">{t('billing.due')}</th>
                  <th className="py-2 pr-3">{t('billing.method')}</th>
                </tr>
              </thead>
              <tbody>
                {data.rows.map((r) => (
                  <tr key={r.bill_id} className="border-b last:border-0">
                    <td className="py-2 pr-3 whitespace-nowrap">{r.receipt_no}</td>
                    <td className="py-2 pr-3 whitespace-nowrap">{fmtDate(r.visit_date)}</td>
                    <td className="py-2 pr-3">
                      <Link to={`/patients/${r.patient_id}`} className="text-brand hover:underline print:text-black">
                        {r.patient_name}
                      </Link>{' '}
                      <span className="text-xs text-slate-500">{r.patient_code}</span>
                    </td>
                    <td className="py-2 pr-3">{t(`visits.type.${r.visit_type}`)}</td>
                    <td className="py-2 pr-3 text-right">{taka(r.fee)}</td>
                    <td className="py-2 pr-3 text-right">{r.discount > 0 ? taka(r.discount) : '—'}</td>
                    <td className="py-2 pr-3 text-right">{taka(r.paid)}</td>
                    <td className={`py-2 pr-3 text-right ${r.due > 0 ? 'font-medium text-amber-700' : ''}`}>
                      {r.due > 0 ? taka(r.due) : '—'}
                    </td>
                    <td className="py-2 pr-3">{t(`billing.methods.${r.method}`)}</td>
                  </tr>
                ))}
              </tbody>
              {tot && (
                <tfoot className="border-t font-semibold">
                  <tr>
                    <td className="py-2 pr-3" colSpan={4}>
                      {t('billing.total')}
                    </td>
                    <td className="py-2 pr-3 text-right">{taka(tot.fee)}</td>
                    <td className="py-2 pr-3 text-right">{taka(tot.discount)}</td>
                    <td className="py-2 pr-3 text-right">{taka(tot.paid)}</td>
                    <td className="py-2 pr-3 text-right">{taka(tot.due)}</td>
                    <td />
                  </tr>
                </tfoot>
              )}
            </table>
          </div>
        )}
      </Card>
    </div>
  )
}
