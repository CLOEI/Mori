import { useDeferredValue, useMemo, useState } from 'react'
import { BookOpen, Code2, Search, Sparkles } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { LUA_DOC_SECTIONS, LUA_DOC_VERSION, type DocSection } from '@/lib/lua-docs'
import { cn } from '@/lib/utils'

type FilteredSection = DocSection & {
  matchedEntries: NonNullable<DocSection['entries']>
  matchedTables: NonNullable<DocSection['tables']>
}

function matchesText(haystack: string, term: string) {
  return haystack.toLowerCase().includes(term)
}

export function DocsPage() {
  const [q, setQ] = useState('')
  const deferredQuery = useDeferredValue(q)

  const filtered = useMemo<FilteredSection[]>(() => {
    const term = deferredQuery.trim().toLowerCase()

    return LUA_DOC_SECTIONS
      .map((section) => {
        const entries = section.entries ?? []
        const tables = section.tables ?? []

        if (!term) {
          return {
            ...section,
            matchedEntries: entries,
            matchedTables: tables,
          }
        }

        const matchedEntries = entries.filter((entry) =>
          matchesText(
            [
              entry.name,
              entry.signature,
              entry.description,
              ...(entry.details ?? []),
            ]
              .filter(Boolean)
              .join(' '),
            term,
          ),
        )

        const matchedTables = tables.filter((table) =>
          matchesText(
            [table.title, ...table.columns, ...table.rows.flat()].join(' '),
            term,
          ),
        )

        const sectionMatch = matchesText(
          [section.title, section.summary, section.example].filter(Boolean).join(' '),
          term,
        )

        return {
          ...section,
          matchedEntries: sectionMatch ? entries : matchedEntries,
          matchedTables: sectionMatch ? tables : matchedTables,
        }
      })
      .filter((section) => section.matchedEntries.length > 0 || section.matchedTables.length > 0)
  }, [deferredQuery])

  const totalEntries = useMemo(
    () => LUA_DOC_SECTIONS.reduce((sum, section) => sum + (section.entries?.length ?? 0), 0),
    [],
  )

  return (
    <div className="h-full overflow-hidden bg-background">
      <div className="border-b border-border bg-card/80 backdrop-blur">
        <div className="mx-auto flex max-w-7xl flex-col gap-5 px-6 py-6">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
            <div className="space-y-3">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="outline" className="gap-1.5">
                  <BookOpen className="size-3" />
                  Lua API
                </Badge>
                <Badge variant="secondary">v{LUA_DOC_VERSION}</Badge>
                <Badge variant="outline">Synced to LUA.md</Badge>
              </div>
              <div className="space-y-2">
                <h1 className="text-2xl font-semibold tracking-tight text-foreground">
                  Mori scripting reference
                </h1>
                <p className="max-w-3xl text-sm leading-6 text-muted-foreground">
                  Search the current Lua contract, browse object models, and inspect examples
                  without leaving the dashboard.
                </p>
              </div>
            </div>

            <div className="grid grid-cols-3 gap-3 lg:min-w-[22rem]">
              <StatCard label="Sections" value={String(LUA_DOC_SECTIONS.length)} icon={BookOpen} />
              <StatCard label="Functions" value={String(totalEntries)} icon={Code2} />
              <StatCard label="Search hits" value={String(filtered.length)} icon={Sparkles} />
            </div>
          </div>

          <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
            <div className="relative w-full max-w-xl">
              <Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search functions, fields, tables, or examples..."
                value={q}
                onChange={(e) => setQ(e.target.value)}
                className="h-10 rounded-xl border-border/70 bg-background pl-10 text-sm"
              />
            </div>
            {q.trim() ? (
              <Button size="sm" variant="ghost" onClick={() => setQ('')}>
                Clear search
              </Button>
            ) : (
              <p className="text-xs text-muted-foreground">
                Covers globals, events, bot actions, HTTP, webhooks, and reference tables.
              </p>
            )}
          </div>
        </div>
      </div>

      <div className="mx-auto grid h-[calc(100%-14.5rem)] max-w-7xl xl:grid-cols-[260px_minmax(0,1fr)]">
        <aside className="hidden border-r border-border/70 bg-card/40 xl:block">
          <ScrollArea className="h-full">
            <div className="space-y-2 p-4">
              <p className="px-3 text-[10px] font-semibold uppercase tracking-[0.26em] text-muted-foreground">
                Jump to section
              </p>
              {filtered.map((section) => (
                <button
                  key={section.id}
                  onClick={() =>
                    document.getElementById(section.id)?.scrollIntoView({
                      behavior: 'smooth',
                      block: 'start',
                    })
                  }
                  className="w-full rounded-xl border border-transparent px-3 py-2 text-left transition-colors hover:border-border hover:bg-background"
                >
                  <p className="text-sm font-medium text-foreground">{section.title}</p>
                  <p className="mt-1 line-clamp-2 text-xs leading-5 text-muted-foreground">
                    {section.summary}
                  </p>
                </button>
              ))}
            </div>
          </ScrollArea>
        </aside>

        <ScrollArea className="h-full">
          <div className="mx-auto flex max-w-5xl flex-col gap-5 px-6 py-6">
            {filtered.map((section) => (
              <section
                key={section.id}
                id={section.id}
                className="overflow-hidden rounded-3xl border border-border/70 bg-card shadow-sm"
              >
                <div className="border-b border-border/70 bg-gradient-to-r from-muted/70 via-card to-card px-6 py-5">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="space-y-2">
                      <p className="text-[10px] font-semibold uppercase tracking-[0.26em] text-muted-foreground">
                        {section.title}
                      </p>
                      <h2 className="text-xl font-semibold tracking-tight text-foreground">
                        {section.summary}
                      </h2>
                    </div>
                    <Badge variant="outline">
                      {section.matchedEntries.length + section.matchedTables.length} blocks
                    </Badge>
                  </div>
                </div>

                <div className="space-y-5 p-6">
                  {section.matchedEntries.length > 0 && (
                    <div className="grid gap-3">
                      {section.matchedEntries.map((entry) => (
                        <article
                          key={`${section.id}-${entry.name}-${entry.signature ?? 'entry'}`}
                          className="rounded-2xl border border-border/70 bg-background/70 p-4"
                        >
                          <div className="space-y-2">
                            <div className="flex flex-wrap items-center gap-2">
                              <Badge variant="secondary">{entry.name}</Badge>
                              {entry.signature && (
                                <code className="rounded-lg bg-muted px-2.5 py-1 font-mono text-xs text-foreground">
                                  {entry.signature}
                                </code>
                              )}
                            </div>
                            <p className="text-sm leading-6 text-foreground">{entry.description}</p>
                            {entry.details && entry.details.length > 0 && (
                              <div className="flex flex-wrap gap-2">
                                {entry.details.map((detail) => (
                                  <span
                                    key={detail}
                                    className="rounded-full border border-border bg-card px-2.5 py-1 text-xs text-muted-foreground"
                                  >
                                    {detail}
                                  </span>
                                ))}
                              </div>
                            )}
                          </div>
                        </article>
                      ))}
                    </div>
                  )}

                  {section.matchedTables.map((table) => (
                    <div key={`${section.id}-${table.title}`} className="overflow-hidden rounded-2xl border border-border/70">
                      <div className="border-b border-border/70 bg-muted/40 px-4 py-3">
                        <h3 className="text-sm font-semibold text-foreground">{table.title}</h3>
                      </div>
                      <Table>
                        <TableHeader>
                          <TableRow>
                            {table.columns.map((column) => (
                              <TableHead key={column} className="text-xs font-semibold">
                                {column}
                              </TableHead>
                            ))}
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {table.rows.map((row, rowIndex) => (
                            <TableRow key={`${table.title}-${rowIndex}`}>
                              {row.map((cell, cellIndex) => (
                                <TableCell
                                  key={`${table.title}-${rowIndex}-${cellIndex}`}
                                  className={cn(
                                    'align-top text-xs leading-6',
                                    cellIndex === 0 ? 'font-mono text-foreground' : 'text-muted-foreground',
                                  )}
                                >
                                  {cell}
                                </TableCell>
                              ))}
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </div>
                  ))}

                  {section.example && (
                    <div className="overflow-hidden rounded-2xl border border-border/70 bg-zinc-950 text-zinc-50">
                      <div className="border-b border-white/10 px-4 py-2 text-xs font-semibold uppercase tracking-[0.22em] text-zinc-400">
                        Example
                      </div>
                      <pre className="overflow-x-auto p-4 text-xs leading-6">
                        <code>{section.example}</code>
                      </pre>
                    </div>
                  )}
                </div>
              </section>
            ))}

            {filtered.length === 0 && (
              <div className="rounded-3xl border border-dashed border-border bg-card/60 px-6 py-16 text-center">
                <p className="text-lg font-medium text-foreground">No docs matched your search.</p>
                <p className="mt-2 text-sm text-muted-foreground">
                  Try Bot, Tile, HttpClient, Webhook, or a method like collectObject.
                </p>
              </div>
            )}
          </div>
        </ScrollArea>
      </div>
    </div>
  )
}

function StatCard({
  label,
  value,
  icon: Icon,
}: {
  label: string
  value: string
  icon: typeof BookOpen
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-background/80 px-4 py-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-muted-foreground">
            {label}
          </p>
          <p className="mt-1 text-lg font-semibold text-foreground">{value}</p>
        </div>
        <div className="rounded-xl border border-border/70 bg-card p-2 text-muted-foreground">
          <Icon className="size-4" />
        </div>
      </div>
    </div>
  )
}
