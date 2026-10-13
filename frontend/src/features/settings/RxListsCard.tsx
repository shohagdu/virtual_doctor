import { useEffect, useRef, useState } from 'react'
import { useLocation } from 'react-router'
import { useTranslation } from 'react-i18next'
import { ListChecks, Plus, Search, X } from 'lucide-react'
import type { ClinicSettings, RxListKey } from '../../api/types'
import { Card } from '../../components/Card'

const KEYS: { key: RxListKey; label: string }[] = [
  { key: 'complaints', label: 'rx.complaints' },
  { key: 'examination', label: 'rx.examination' },
  { key: 'investigations', label: 'rx.investigations' },
  { key: 'diagnosis', label: 'rx.diagnosis' },
  { key: 'advice', label: 'rx.advice' },
  { key: 'next_plan', label: 'rx.nextPlan' },
  { key: 'chronic_conditions', label: 'profile.chronic' },
  { key: 'companies', label: 'medicines.companies' },
]

/**
 * Settings → Prescription lists: the items each prescription section offers in its
 * dropdown. Type one item, or paste many (one per line); saved with the form.
 */
export function RxListsCard({
  value,
  onChange,
}: {
  value: ClinicSettings['rx_lists']
  onChange: (v: ClinicSettings['rx_lists']) => void
}) {
  const { t } = useTranslation()
  // /settings#companies (or any list key) opens that tab and scrolls here.
  const { hash } = useLocation()
  const linked = KEYS.find((k) => `#${k.key}` === hash)?.key
  const [tab, setTab] = useState<RxListKey>(linked ?? 'complaints')
  const ref = useRef<HTMLDivElement>(null)
  useEffect(() => {
    if (linked) ref.current?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }, [linked])
  const [text, setText] = useState('')
  const [filter, setFilter] = useState('')
  const items = value[tab]
  const shown = filter.trim() ? items.filter((x) => x.toLowerCase().includes(filter.trim().toLowerCase())) : items

  const add = (raw: string) => {
    const fresh = raw
      .split(/\r?\n/)
      .map((x) => x.trim())
      .filter((x) => x && !items.some((y) => y.toLowerCase() === x.toLowerCase()))
    const unique = fresh.filter((x, i) => fresh.findIndex((y) => y.toLowerCase() === x.toLowerCase()) === i)
    if (unique.length) onChange({ ...value, [tab]: [...items, ...unique] })
    setText('')
  }
  const remove = (x: string) => onChange({ ...value, [tab]: items.filter((y) => y !== x) })

  return (
    <div ref={ref} className="scroll-mt-20">
    <Card
      title={
        <span className="flex items-center gap-2">
          <ListChecks className="h-5 w-5 text-brand" /> {t('lists.title')}
        </span>
      }
    >
      <p className="mb-3 text-sm text-slate-500">{t('lists.help')}</p>
      <div className="mb-4 flex gap-1 overflow-x-auto border-b border-slate-100" role="tablist">
        {KEYS.map(({ key, label }) => (
          <button
            key={key}
            type="button"
            role="tab"
            aria-selected={tab === key}
            onClick={() => {
              setTab(key)
              setFilter('')
            }}
            className={`-mb-px flex shrink-0 items-center gap-1.5 border-b-2 px-3 py-2 text-sm font-medium whitespace-nowrap ${
              tab === key ? 'border-brand text-brand' : 'border-transparent text-slate-500 hover:text-slate-800'
            }`}
          >
            {t(label)}
            <span className={`rounded-full px-1.5 text-xs ${tab === key ? 'bg-brand-50' : 'bg-slate-100'}`}>{value[key].length}</span>
          </button>
        ))}
      </div>

      <div className="flex flex-wrap gap-2">
        <textarea
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault()
              add(text)
            }
          }}
          rows={1}
          placeholder={t('lists.addPlaceholder')}
          className="min-h-[38px] min-w-0 flex-1 resize-y rounded-lg border px-3 py-2 text-sm"
          aria-label={t('lists.addPlaceholder')}
        />
        <button type="button" onClick={() => add(text)} disabled={!text.trim()} className="btn btn-secondary">
          <Plus className="h-4 w-4" /> {t('lists.add')}
        </button>
      </div>
      <p className="mt-1 text-xs text-slate-400">{t('lists.pasteHint')}</p>

      {items.length > 12 && (
        <div className="relative mt-3 max-w-xs">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-slate-400" />
          <input
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder={t('lists.filter')}
            className="w-full rounded-lg border py-1.5 pr-2 pl-8 text-sm"
          />
        </div>
      )}

      <ul className="mt-3 flex flex-wrap gap-1.5">
        {shown.map((x) => (
          <li key={x} className="inline-flex items-center gap-1 rounded-full bg-slate-50 py-1 pr-1.5 pl-3 text-sm text-slate-700 ring-1 ring-slate-200">
            {x}
            <button
              type="button"
              onClick={() => remove(x)}
              className="rounded-full p-0.5 text-slate-400 hover:bg-red-50 hover:text-red-600"
              aria-label={`${t('lists.remove')} ${x}`}
            >
              <X className="h-3.5 w-3.5" />
            </button>
          </li>
        ))}
        {items.length === 0 && <li className="text-sm text-slate-400">{t('lists.empty')}</li>}
      </ul>
      <p className="mt-3 text-xs text-slate-500">{t('lists.saveHint')}</p>
    </Card>
    </div>
  )
}
