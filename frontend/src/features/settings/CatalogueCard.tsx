import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Card } from '../../components/Card'
import { useCatalogue, useImportCatalogue } from '../rxtools/api'

const HEADER = 'brand,generic,strength,form,company,price\n'

/**
 * Medicine catalogue (roadmap 2.2): import the clinic's brand list from a CSV file.
 * Brands with generics make suggestions faster and let the allergy check match by
 * generic and class. No data is bundled; the clinic supplies its own list.
 */
export function CatalogueCard() {
  const { t } = useTranslation()
  const { data } = useCatalogue()
  const imp = useImportCatalogue()
  const [name, setName] = useState('')
  const templateHref = `data:text/csv;charset=utf-8,${encodeURIComponent(HEADER)}`

  return (
    <Card title={`💊 ${t('rxtools.catalogue')}`}>
      <div className="space-y-2 text-sm">
        {data && <p>{t('rxtools.catalogueSize', { clinic: data.clinic_rows, shared: data.shared_rows })}</p>}
        <p className="text-slate-500">{t('rxtools.catalogueHelp')}</p>
        <div className="flex flex-wrap items-center gap-2">
          <label className="cursor-pointer rounded bg-brand px-3 py-1.5 text-white">
            📥 {t('rxtools.importCsv')}
            <input
              type="file"
              accept=".csv,text/csv"
              className="hidden"
              onChange={async (e) => {
                const file = e.target.files?.[0]
                e.target.value = ''
                if (!file) return
                setName(file.name)
                imp.mutate(await file.text())
              }}
            />
          </label>
          <a href={templateHref} download="medicines-template.csv" className="text-brand underline">
            {t('rxtools.csvTemplate')}
          </a>
        </div>
        {imp.isPending && <p className="text-slate-500">{t('rxtools.importing', { name })}</p>}
        {imp.isError && <p className="text-red-600">{imp.error.message}</p>}
        {imp.data && (
          <div className="rounded bg-panel p-2">
            <p className="text-green-700">✔ {t('rxtools.imported', { added: imp.data.added, updated: imp.data.updated })}</p>
            {imp.data.problems.length > 0 && (
              <ul className="mt-1 text-xs text-amber-800">
                {imp.data.problems.map((p) => (
                  <li key={`${p.line}-${p.message}`}>
                    {t('rxtools.line', { n: p.line })}: {p.message}
                  </li>
                ))}
                {imp.data.more_problems > 0 && <li>{t('rxtools.moreProblems', { count: imp.data.more_problems })}</li>}
              </ul>
            )}
          </div>
        )}
      </div>
    </Card>
  )
}
