import { useState, type FormEvent } from 'react'
import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import { Pill } from 'lucide-react'
import type { Medicine } from '../../api/types'
import { useSaveMedicine } from './api'

const input = 'mt-1 w-full rounded-lg border px-3 py-2'

/** Add or edit one catalogue medicine. */
export function MedicineDialog({
  medicine,
  forms,
  companies,
  onClose,
}: {
  medicine?: Medicine
  forms: string[]
  /** From Settings → Prescription lists → Companies. */
  companies: string[]
  onClose: () => void
}) {
  const { t } = useTranslation()
  const save = useSaveMedicine()
  const [f, setF] = useState({
    brand: medicine?.brand ?? '',
    generic: medicine?.generic ?? '',
    strength: medicine?.strength ?? '',
    form: medicine?.form ?? '',
    company: medicine?.company ?? '',
    price: medicine?.price != null ? String(medicine.price) : '',
  })
  const set = (patch: Partial<typeof f>) => setF({ ...f, ...patch })

  const submit = (e: FormEvent) => {
    e.preventDefault()
    save.mutate(
      {
      id: medicine?.id,
      brand: f.brand,
      generic: f.generic,
      strength: f.strength || null,
      form: f.form || null,
      company: f.company || null,
      price: f.price.trim() === '' ? null : Number(f.price),
      },
      { onSuccess: onClose },
    )
  }

  return (
    <div className="overlay">
      <form onSubmit={submit} className="dialog w-full max-w-lg space-y-4">
        <h2 className="flex items-center gap-2 text-lg font-semibold">
          <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-brand-50 text-brand">
            <Pill className="h-4 w-4" />
          </span>
          {medicine ? t('medicines.edit') : t('medicines.add')}
        </h2>
        <div className="grid gap-3 sm:grid-cols-2">
          <label className="text-sm font-medium">
            {t('medicines.brand')} *
            <input value={f.brand} onChange={(e) => set({ brand: e.target.value })} required maxLength={150} autoFocus className={input} placeholder="Napa" />
          </label>
          <label className="text-sm font-medium">
            {t('medicines.generic')} *
            <input value={f.generic} onChange={(e) => set({ generic: e.target.value })} required maxLength={255} className={input} placeholder="Paracetamol" />
          </label>
          <label className="text-sm font-medium">
            {t('medicines.strength')}
            <input value={f.strength} onChange={(e) => set({ strength: e.target.value })} maxLength={100} className={input} placeholder="500 mg" />
          </label>
          <label className="text-sm font-medium">
            {t('medicines.form')}
            <input value={f.form} onChange={(e) => set({ form: e.target.value })} list="medicine-forms" maxLength={50} className={input} placeholder="Tablet" />
            <datalist id="medicine-forms">
              {forms.map((x) => (
                <option key={x} value={x} />
              ))}
            </datalist>
          </label>
          <label className="text-sm font-medium">
            <span className="flex items-center justify-between">
              {t('medicines.company')}
              <Link to="/settings#companies" className="text-xs font-normal text-brand hover:underline">
                {t('medicines.manageCompanies')}
              </Link>
            </span>
            <select value={f.company} onChange={(e) => set({ company: e.target.value })} className={`${input} bg-white`}>
              <option value="">—</option>
              {/* Keep a company the medicine already has, even if it was removed from the list. */}
              {[...companies, ...(f.company && !companies.includes(f.company) ? [f.company] : [])].map((c) => (
                <option key={c} value={c}>
                  {c}
                </option>
              ))}
            </select>
          </label>
          <label className="text-sm font-medium">
            {t('medicines.price')} (৳)
            <input type="number" min={0} step="0.01" value={f.price} onChange={(e) => set({ price: e.target.value })} className={input} />
          </label>
        </div>
        <p className="text-xs text-slate-500">{t('medicines.genericHint')}</p>
        {save.isError && <p className="text-sm text-red-600">{save.error.message}</p>}
        <div className="flex justify-end gap-2">
          <button type="button" onClick={onClose} className="btn btn-secondary">
            {t('patients.cancel')}
          </button>
          <button disabled={save.isPending} className="btn btn-primary">
            {t('patients.save')}
          </button>
        </div>
      </form>
    </div>
  )
}
