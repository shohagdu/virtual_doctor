import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import type { Vitals } from '../../api/types'
import { useSaveVitals, useVisitVitals } from './api'

type Field = Exclude<keyof Vitals, 'bmi'>

const FIELDS: { key: Field; label: string; unit: string; step: string }[] = [
  { key: 'bp_sys', label: 'BP systolic', unit: 'mmHg', step: '1' },
  { key: 'bp_dia', label: 'BP diastolic', unit: 'mmHg', step: '1' },
  { key: 'pulse', label: 'Pulse', unit: '/min', step: '1' },
  { key: 'temp', label: 'Temp', unit: '°F', step: '0.1' },
  { key: 'weight', label: 'Weight', unit: 'kg', step: '0.1' },
  { key: 'height', label: 'Height', unit: 'cm', step: '0.1' },
  { key: 'spo2', label: 'SpO₂', unit: '%', step: '1' },
  { key: 'rbs', label: 'RBS', unit: 'mmol/L', step: '0.1' },
]

type Form = Record<Field, string>

function toForm(v: Vitals | null | undefined): Form {
  const f = {} as Form
  for (const { key } of FIELDS) f[key] = v?.[key]?.toString() ?? ''
  return f
}

/** Records the visit's vitals (§4: Doctor and Assistant). Re-saving corrects them. */
export function VitalsDialog({ visitId, patientId, onClose }: { visitId: number; patientId: number; onClose: () => void }) {
  const { data, isLoading } = useVisitVitals(visitId)
  if (isLoading) return null
  return <VitalsForm visitId={visitId} patientId={patientId} initial={data ?? null} onClose={onClose} />
}

function VitalsForm({
  visitId,
  patientId,
  initial,
  onClose,
}: {
  visitId: number
  patientId: number
  initial: Vitals | null
  onClose: () => void
}) {
  const { t } = useTranslation()
  const save = useSaveVitals(visitId, patientId)
  const [form, setForm] = useState<Form>(() => toForm(initial))

  const num = (s: string) => (s.trim() === '' ? null : Number(s))
  const weight = num(form.weight)
  const height = num(form.height)
  const bmi = weight && height ? Math.round((weight / (height / 100) ** 2) * 10) / 10 : null

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    const v = { bmi: null } as Vitals
    for (const { key } of FIELDS) v[key] = num(form[key])
    await save.mutateAsync(v)
    onClose()
  }

  return (
    <div className="fixed inset-0 z-30 flex items-start justify-center overflow-y-auto bg-black/40 p-4">
      <form onSubmit={submit} className="w-full max-w-lg space-y-4 rounded-lg bg-white p-6 shadow-lg">
        <h2 className="text-lg font-semibold">{t('vitals.record')}</h2>
        <div className="grid grid-cols-2 gap-3">
          {FIELDS.map((f) => (
            <label key={f.key} className="text-sm">
              {f.label} <span className="text-slate-400">({f.unit})</span>
              <input
                type="number"
                inputMode="decimal"
                step={f.step}
                value={form[f.key]}
                onChange={(e) => setForm({ ...form, [f.key]: e.target.value })}
                className="mt-1 w-full rounded border px-3 py-1.5"
              />
            </label>
          ))}
        </div>
        {bmi !== null && Number.isFinite(bmi) && (
          <p className="text-sm">
            <b>BMI:</b> {bmi}
          </p>
        )}
        {save.isError && <p className="text-sm text-red-600">{save.error.message}</p>}
        <div className="flex justify-end gap-2">
          <button type="button" onClick={onClose} className="rounded border px-4 py-2">
            {t('patients.cancel')}
          </button>
          <button disabled={save.isPending} className="rounded bg-brand px-4 py-2 text-white disabled:opacity-60">
            {t('patients.save')}
          </button>
        </div>
      </form>
    </div>
  )
}
