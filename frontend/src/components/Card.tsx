import type { ReactNode } from 'react'

export function Card({ title, actions, children }: { title?: ReactNode; actions?: ReactNode; children: ReactNode }) {
  return (
    <section className="rounded-lg bg-white shadow-sm">
      {(title || actions) && (
        <div className="flex flex-wrap items-center gap-2 border-b px-4 py-3">
          <h2 className="font-semibold">{title}</h2>
          <div className="ml-auto flex flex-wrap gap-2">{actions}</div>
        </div>
      )}
      <div className="p-4">{children}</div>
    </section>
  )
}
