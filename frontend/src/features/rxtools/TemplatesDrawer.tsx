import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { PrescriptionBody, RxTemplate } from '../../api/types'
import { medicineName } from '../prescription/medicine'
import { useDeleteTemplate, useSaveTemplate, useTemplates } from './api'

/**
 * 📋 Prescription templates (roadmap 2.1): apply a saved set of sections and medicines
 * in one click, or save the current prescription as a template.
 */
export function TemplatesDrawer({
  current,
  onApply,
  onClose,
}: {
  current: PrescriptionBody
  onApply: (body: PrescriptionBody) => void
  onClose: () => void
}) {
  const { t } = useTranslation()
  const { data } = useTemplates()
  const save = useSaveTemplate()
  const del = useDeleteTemplate()
  const [name, setName] = useState(() => current.diagnosis[0] ?? '')
  const [confirmDelete, setConfirmDelete] = useState<number | null>(null)
  const empty =
    !current.medicines.some((m) => m.brand.trim()) &&
    [current.complaints, current.diagnosis, current.investigations, current.advice, current.next_plan].every((l) => l.length === 0)

  return (
    <div className="fixed inset-0 z-30 print:hidden" role="dialog" aria-label={t('rxtools.templates')}>
      <div className="absolute inset-0 bg-black/30" onClick={onClose} />
      <aside className="absolute inset-y-0 right-0 flex w-full max-w-md flex-col bg-white shadow-xl">
        <div className="flex items-center border-b px-4 py-3">
          <h2 className="font-semibold">📋 {t('rxtools.templates')}</h2>
          <button onClick={onClose} className="ml-auto px-2 text-lg" aria-label={t('common.close')}>
            ✕
          </button>
        </div>
        <div className="flex-1 space-y-4 overflow-y-auto p-4 text-sm">
          <form
            onSubmit={async (e) => {
              e.preventDefault()
              await save.mutateAsync({ name, body: current })
            }}
            className="space-y-2 rounded bg-panel p-3"
          >
            <div className="font-semibold">{t('rxtools.saveCurrent')}</div>
            <div className="flex gap-2">
              <input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t('rxtools.templateName')}
                maxLength={100}
                className="min-w-0 flex-1 rounded border px-2 py-1"
                aria-label={t('rxtools.templateName')}
              />
              <button disabled={!name.trim() || empty || save.isPending} className="rounded bg-brand px-3 py-1 text-white disabled:opacity-50">
                {t('patients.save')}
              </button>
            </div>
            <p className="text-xs text-slate-500">{t('rxtools.saveHint')}</p>
            {save.isError && <p className="text-red-600">{save.error.message}</p>}
            {save.isSuccess && <p className="text-green-700">✔ {t('settings.saved')}</p>}
          </form>

          {!data ? (
            <p className="text-slate-500">{t('common.loading')}</p>
          ) : data.length === 0 ? (
            <p className="text-slate-500">{t('rxtools.noTemplates')}</p>
          ) : (
            <ul className="divide-y">
              {data.map((tpl) => (
                <TemplateRow
                  key={tpl.id}
                  tpl={tpl}
                  onApply={() => onApply(tpl.body)}
                  confirming={confirmDelete === tpl.id}
                  onDelete={() => (confirmDelete === tpl.id ? del.mutate(tpl.id) : setConfirmDelete(tpl.id))}
                />
              ))}
            </ul>
          )}
        </div>
      </aside>
    </div>
  )
}

function TemplateRow({
  tpl,
  onApply,
  onDelete,
  confirming,
}: {
  tpl: RxTemplate
  onApply: () => void
  onDelete: () => void
  confirming: boolean
}) {
  const { t } = useTranslation()
  const b = tpl.body
  return (
    <li className="space-y-1 py-2">
      <div className="flex items-center gap-2">
        <b>{tpl.name}</b>
        <button onClick={onApply} className="ml-auto rounded bg-brand px-2 py-0.5 text-xs text-white">
          {t('rxtools.apply')}
        </button>
        <button onClick={onDelete} className={`px-1 text-xs ${confirming ? 'rounded bg-red-600 px-2 text-white' : 'text-red-600'}`}>
          {confirming ? t('rxtools.confirmDelete') : '🗑'}
        </button>
      </div>
      {b.diagnosis.length > 0 && <div className="text-slate-600">{b.diagnosis.join(', ')}</div>}
      {b.medicines.length > 0 && <div className="text-xs text-slate-500">℞ {b.medicines.map(medicineName).join(' · ')}</div>}
    </li>
  )
}
