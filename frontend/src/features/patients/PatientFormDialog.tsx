import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate } from 'react-router'
import { useTranslation } from 'react-i18next'
import { differenceInYears } from 'date-fns'
import type { Patient } from '../../api/types'
import { ageText } from '../../utils/age'
import { useCreatePatient, useUpdatePatient } from './api'
import { patientSchema, type PatientForm } from './schema'

const input = 'mt-1 w-full rounded-md border px-3 py-2'

function toForm(p: Patient): PatientForm {
  return {
    title: p.title ?? '',
    name_en: p.name_en,
    name_bn: p.name_bn ?? '',
    phone: p.phone,
    // An estimated DOB is edited as an age, an exact one as a date.
    dob: p.dob_estimated ? '' : p.dob,
    age_years: p.dob_estimated ? differenceInYears(new Date(), new Date(p.dob)) : undefined,
    sex: p.sex,
    blood_group: p.blood_group ?? '',
    address: p.address ?? '',
    district: p.district ?? '',
  }
}

/**
 * Registers a new patient, or edits one when `patient` is given. After registering it
 * opens the new profile, unless `onCreated` takes the patient (e.g. to book them in).
 */
export function PatientFormDialog({
  patient,
  onClose,
  onCreated,
}: {
  patient?: Patient
  onClose: () => void
  onCreated?: (p: Patient) => void
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const create = useCreatePatient()
  const update = useUpdatePatient(patient?.id ?? 0)
  const mutation = patient ? update : create
  const [duplicates, setDuplicates] = useState<Patient[]>([])
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<PatientForm>({
    resolver: zodResolver(patientSchema),
    defaultValues: patient ? toForm(patient) : { sex: 'M' },
  })

  const save = (force: boolean) =>
    handleSubmit(async (form) => {
      if (patient) {
        await update.mutateAsync(form)
        onClose()
        return
      }
      const res = await create.mutateAsync({ form, force })
      if (res.patient) {
        onClose()
        if (onCreated) onCreated(res.patient)
        else navigate(`/patients/${res.patient.id}`)
      } else {
        setDuplicates(res.duplicates)
      }
    })

  const err = (name: keyof PatientForm) =>
    errors[name] && <span className="text-xs text-red-600">{errors[name]?.message}</span>

  return (
    <div className="overlay">
      <form onSubmit={save(false)} className="w-full max-w-2xl space-y-4 dialog">
        <h2 className="text-lg font-semibold">{patient ? t('patients.edit') : t('patients.new')}</h2>
        <div className="grid gap-3 sm:grid-cols-2">
          <label className="text-sm">
            Title
            <select {...register('title')} className={input}>
              <option value="">—</option>
              {['Mr', 'Ms', 'Mrs', 'Baby', 'Master'].map((v) => (
                <option key={v}>{v}</option>
              ))}
            </select>
          </label>
          <label className="text-sm">
            Full Name *
            <input {...register('name_en')} className={input} />
            {err('name_en')}
          </label>
          <label className="text-sm">
            নাম (বাংলা)
            <input {...register('name_bn')} className={input} />
          </label>
          <label className="text-sm">
            Mobile *
            <input {...register('phone')} inputMode="tel" placeholder="01XXXXXXXXX" className={input} />
            {err('phone')}
          </label>
          <label className="text-sm">
            Age (years)
            <input
              {...register('age_years', { setValueAs: (v) => (v === '' || v === undefined ? undefined : Number(v)) })}
              type="number"
              className={input}
            />
            {err('age_years')}
          </label>
          <label className="text-sm">
            or Date of Birth
            <input {...register('dob')} type="date" className={input} />
          </label>
          <label className="text-sm">
            Sex *
            <select {...register('sex')} className={input}>
              <option value="M">Male</option>
              <option value="F">Female</option>
              <option value="O">Other</option>
            </select>
          </label>
          <label className="text-sm">
            Blood Group
            <select {...register('blood_group')} className={input}>
              <option value="">—</option>
              {['A+', 'A-', 'B+', 'B-', 'AB+', 'AB-', 'O+', 'O-'].map((v) => (
                <option key={v}>{v}</option>
              ))}
            </select>
          </label>
          <label className="text-sm">
            Address / Area
            <input {...register('address')} className={input} />
          </label>
          <label className="text-sm">
            District
            <input {...register('district')} className={input} />
          </label>
        </div>

        {duplicates.length > 0 && (
          <div className="rounded-md border border-amber-300 bg-amber-50 p-3 text-sm">
            <p className="mb-2 font-medium">{t('patients.duplicates')}</p>
            <ul className="space-y-1">
              {duplicates.map((p) => (
                <li key={p.id}>
                  <button
                    type="button"
                    className="text-brand underline"
                    onClick={() => {
                      onClose()
                      if (onCreated) onCreated(p)
                      else navigate(`/patients/${p.id}`)
                    }}
                  >
                    {p.patient_code} — {p.name_en} ({ageText(p.dob)} / {p.sex}) {p.phone}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}

        {mutation.isError && <p className="text-sm text-red-600">{mutation.error.message}</p>}

        <div className="flex flex-wrap justify-end gap-2">
          <button type="button" onClick={onClose} className="btn btn-secondary">
            {t('patients.cancel')}
          </button>
          {duplicates.length > 0 ? (
            <button type="button" onClick={save(true)} className="rounded-md bg-amber-600 px-4 py-2 text-white">
              {t('patients.createAnyway')}
            </button>
          ) : (
            <button disabled={mutation.isPending} className="btn btn-primary">
              {t('patients.save')}
            </button>
          )}
        </div>
      </form>
    </div>
  )
}
