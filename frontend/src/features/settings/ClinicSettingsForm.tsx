import { useState, type FormEvent } from 'react'
import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { ChamberSession, ClinicSettings, Lang, SmsTemplate, VisitType } from '../../api/types'
import { Card } from '../../components/Card'
import { smsSegments } from '../../utils/sms'
import { useClinicSettings, useSaveClinicSettings } from './api'
import { useAiStatus } from '../ai/api'

const input = 'mt-1 w-full rounded border px-3 py-2'
const VISIT_TYPES: VisitType[] = ['new', 'followup', 'report']
const TEMPLATES = ['appointment_confirm', 'appointment_reminder', 'followup_reminder'] as const
// Bangladesh week order: Saturday first.
const WEEK = [6, 0, 1, 2, 3, 4, 5]

/** Fees, chamber schedule / slots and SMS (§12). Doctor and admin only. */
export function ClinicSettingsForm() {
  const { data } = useClinicSettings()
  const { t } = useTranslation()
  if (!data) return <p className="text-slate-500">{t('common.loading')}</p>
  return <Form initial={data} />
}

function Form({ initial }: { initial: ClinicSettings }) {
  const { t } = useTranslation()
  const save = useSaveClinicSettings()
  const ai = useAiStatus()
  const [s, setS] = useState(initial)
  const [saved, setSaved] = useState(false)

  const set = (patch: Partial<ClinicSettings>) => {
    setS({ ...s, ...patch })
    setSaved(false)
  }
  const setSms = (patch: Partial<ClinicSettings['sms']>) => set({ sms: { ...s.sms, ...patch } })
  const setAi = (patch: Partial<ClinicSettings['ai']>) => set({ ai: { ...s.ai, ...patch } })
  const setSession = (i: number, patch: Partial<ChamberSession>) =>
    set({ schedule: s.schedule.map((x, j) => (j === i ? { ...x, ...patch } : x)) })
  const setTemplate = (key: (typeof TEMPLATES)[number], lang: Lang, text: string) =>
    setSms({ templates: { ...s.sms.templates, [key]: { ...s.sms.templates[key], [lang]: text } as SmsTemplate } })

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    const result = await save.mutateAsync(s)
    setS(result)
    setSaved(true)
  }

  return (
    <form onSubmit={submit} className="space-y-4 md:col-span-2">
      <div className="grid gap-4 md:grid-cols-2">
        <Card title={t('settings.fees')}>
          <div className="grid grid-cols-3 gap-3">
            {VISIT_TYPES.map((v) => (
              <label key={v} className="text-sm">
                {t(`visits.type.${v}`)} (৳)
                <input
                  type="number"
                  min={0}
                  value={s.fees[v]}
                  onChange={(e) => set({ fees: { ...s.fees, [v]: Number(e.target.value) } })}
                  className={input}
                />
              </label>
            ))}
          </div>
        </Card>

        <Card title={t('settings.appointments')}>
          <div className="grid grid-cols-2 gap-3">
            <label className="text-sm">
              {t('settings.slotMinutes')}
              <input
                type="number"
                min={5}
                max={120}
                value={s.slot_minutes}
                onChange={(e) => set({ slot_minutes: Number(e.target.value) })}
                className={input}
              />
            </label>
            <label className="text-sm">
              {t('settings.dailyLimit')}
              <input
                type="number"
                min={1}
                max={500}
                value={s.daily_limit ?? ''}
                placeholder={t('settings.noLimit')}
                onChange={(e) => set({ daily_limit: e.target.value === '' ? null : Number(e.target.value) })}
                className={input}
              />
            </label>
          </div>
        </Card>
      </div>

      <Card
        title={t('settings.schedule')}
        actions={
          <button
            type="button"
            onClick={() => set({ schedule: [...s.schedule, { weekday: 6, start: '17:00', end: '21:00' }] })}
            className="rounded border px-2 py-1 text-sm text-brand"
          >
            ➕ {t('settings.addSession')}
          </button>
        }
      >
        {s.schedule.length === 0 && <p className="text-sm text-slate-500">{t('settings.noSchedule')}</p>}
        <ul className="space-y-2">
          {s.schedule.map((x, i) => (
            <li key={i} className="flex flex-wrap items-center gap-2 text-sm">
              <select
                value={x.weekday}
                onChange={(e) => setSession(i, { weekday: Number(e.target.value) })}
                className="rounded border bg-white px-2 py-1.5"
                aria-label={t('settings.day')}
              >
                {WEEK.map((d) => (
                  <option key={d} value={d}>
                    {t(`days.${d}`)}
                  </option>
                ))}
              </select>
              <input
                type="time"
                value={x.start}
                onChange={(e) => setSession(i, { start: e.target.value })}
                className="rounded border px-2 py-1"
                aria-label={t('settings.from')}
              />
              –
              <input
                type="time"
                value={x.end}
                onChange={(e) => setSession(i, { end: e.target.value })}
                className="rounded border px-2 py-1"
                aria-label={t('settings.to')}
              />
              <button
                type="button"
                onClick={() => set({ schedule: s.schedule.filter((_, j) => j !== i) })}
                className="px-2 text-red-600"
                aria-label={t('settings.removeSession')}
              >
                ✕
              </button>
            </li>
          ))}
        </ul>
      </Card>

      <Card title={t('settings.sms')}>
        <div className="space-y-3 text-sm">
          <div className="flex flex-wrap gap-x-6 gap-y-2">
            <label className="flex items-center gap-2">
              <input type="checkbox" checked={s.sms.enabled} onChange={(e) => setSms({ enabled: e.target.checked })} />
              {t('settings.smsEnabled')}
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={s.sms.confirm_on_book}
                onChange={(e) => setSms({ confirm_on_book: e.target.checked })}
              />
              {t('settings.smsConfirm')}
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={s.sms.remind_day_before}
                onChange={(e) => setSms({ remind_day_before: e.target.checked })}
              />
              {t('settings.smsRemind')}
            </label>
            <label className="flex items-center gap-2">
              {t('settings.smsLanguage')}
              <select
                value={s.sms.language}
                onChange={(e) => setSms({ language: e.target.value as Lang })}
                className="rounded border bg-white px-2 py-1"
              >
                <option value="bn">বাংলা</option>
                <option value="en">English</option>
              </select>
            </label>
          </div>
          <p className="text-slate-500">{t('settings.smsPlaceholders')}</p>
          {TEMPLATES.map((key) => (
            <fieldset key={key} className="space-y-2 rounded border p-3">
              <legend className="px-1 font-medium">{t(`settings.tpl.${key}`)}</legend>
              {(['bn', 'en'] as const).map((lang) => {
                const text = s.sms.templates[key][lang]
                return (
                  <label key={lang} className="block">
                    <span className="text-slate-500">{lang === 'bn' ? 'বাংলা' : 'English'}</span>
                    <textarea
                      value={text}
                      rows={2}
                      maxLength={480}
                      onChange={(e) => setTemplate(key, lang, e.target.value)}
                      className="mt-1 w-full rounded border px-3 py-2"
                    />
                    <span className="text-xs text-slate-500">
                      {t('settings.smsCount', { chars: [...text].length, parts: smsSegments(text) })}
                    </span>
                  </label>
                )
              })}
            </fieldset>
          ))}
        </div>
      </Card>

      <Card
        title={`🤖 ${t('settings.ai')}`}
        actions={
          <Link to="/ai-usage" className="text-sm text-brand">
            {t('ai.usage')} →
          </Link>
        }
      >
        <div className="space-y-2 text-sm">
          {ai.data && !ai.data.configured && <p className="rounded bg-amber-50 px-3 py-2 text-amber-800">{t('settings.aiNoKey')}</p>}
          <label className="flex items-center gap-2">
            <input type="checkbox" checked={s.ai.enabled} onChange={(e) => setAi({ enabled: e.target.checked })} />
            {t('settings.aiEnabled')}
          </label>
          <label className="flex items-center gap-2">
            <input type="checkbox" checked={s.ai.auto_extract} onChange={(e) => setAi({ auto_extract: e.target.checked })} />
            {t('settings.aiAutoExtract')}
          </label>
          <label className="flex items-center gap-2">
            {t('settings.aiAdviceLanguage')}
            <select
              value={s.ai.advice_language}
              onChange={(e) => setAi({ advice_language: e.target.value as ClinicSettings['ai']['advice_language'] })}
              className="rounded border bg-white px-2 py-1"
            >
              <option value="both">English + বাংলা</option>
              <option value="bn">বাংলা</option>
              <option value="en">English</option>
            </select>
          </label>
          <p className="text-xs text-slate-500">{t('settings.aiPrivacy')}</p>
        </div>
      </Card>

      <div className="flex items-center gap-3">
        <button disabled={save.isPending} className="rounded bg-brand px-4 py-2 text-white disabled:opacity-60">
          {t('patients.save')}
        </button>
        {save.isError && <p className="text-sm text-red-600">{save.error.message}</p>}
        {saved && <p className="text-sm text-green-700">{t('settings.saved')}</p>}
      </div>
    </form>
  )
}
