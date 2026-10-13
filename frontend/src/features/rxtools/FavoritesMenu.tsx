import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { MedicineLine } from '../../api/types'
import { medicineDirections, medicineName } from '../prescription/medicine'
import { useDeleteFavorite, useFavorites } from './api'

/** ★ Favourite medicines with their usual dose; one click adds a line (roadmap 2.3). */
export function FavoritesMenu({ onAdd }: { onAdd: (line: MedicineLine) => void }) {
  const { t } = useTranslation()
  const { data } = useFavorites()
  const del = useDeleteFavorite()
  const [open, setOpen] = useState(false)
  if (!data?.length) return null
  return (
    <div className="relative">
      <button type="button" onClick={() => setOpen(!open)} className="rounded-md border px-2 py-1 text-xs text-amber-700">
        ★ {t('rxtools.favorites')} ({data.length})
      </button>
      {open && (
        <ul className="absolute right-0 z-20 mt-1 max-h-72 w-80 overflow-y-auto rounded-md border bg-white text-sm shadow-lg">
          {data.map((f) => (
            <li key={f.id} className="flex items-start gap-1 border-b last:border-0 hover:bg-slate-50">
              <button
                type="button"
                onClick={() => {
                  onAdd(f.line)
                  setOpen(false)
                }}
                className="min-w-0 flex-1 px-3 py-1.5 text-left"
              >
                <b>{medicineName(f.line)}</b>
                <span className="block text-xs text-slate-500">{medicineDirections(f.line)}</span>
              </button>
              <button
                type="button"
                onClick={() => del.mutate(f.id)}
                className="px-2 py-1.5 text-xs text-red-600"
                aria-label={t('rxtools.removeFavorite')}
              >
                ✕
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
