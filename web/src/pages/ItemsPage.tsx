import { useState, useEffect, useCallback, useRef } from 'react'
import { Search, ChevronLeft, ChevronRight, Loader2 } from 'lucide-react'
import { api, type ItemRecord, type ItemsPage } from '@/lib/api'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table'

export function ItemsPage() {
  const [q, setQ] = useState('')
  const [page, setPage] = useState(1)
  const [data, setData] = useState<ItemsPage | null>(null)
  const [loading, setLoading] = useState(false)
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const fetchItems = useCallback(async (p: number, query: string) => {
    setLoading(true)
    try {
      const result = await api.getItems(p, query)
      setData(result)
    } catch {
      // ignore
    } finally {
      setLoading(false)
    }
  }, [])

  const qRef = useRef(q)
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current)
    const isQueryChange = qRef.current !== q
    qRef.current = q
    if (isQueryChange) {
      setPage(1)
      debounceRef.current = setTimeout(() => fetchItems(1, q), 300)
      return () => { if (debounceRef.current) clearTimeout(debounceRef.current) }
    } else {
      fetchItems(page, q)
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [q, page])

  const totalPages = data ? Math.ceil(data.total / data.page_size) : 0

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="shrink-0 flex items-center gap-3 px-6 py-3 border-b border-border bg-card">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
          <Input
            placeholder="Search by ID or name…"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            className="pl-8 h-8 text-xs"
          />
        </div>
        {loading && <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />}
        {data && !loading && (
          <span className="text-xs text-muted-foreground shrink-0">
            {data.total.toLocaleString()} items
          </span>
        )}
      </div>

      <ScrollArea className="flex-1 min-h-0">
        <Table className="text-xs">
          <TableHeader className="sticky top-0 z-10 bg-card">
            <TableRow className="border-b border-border">
              <TableHead>ID</TableHead>
              <TableHead>Name</TableHead>
              <TableHead>Action</TableHead>
              <TableHead>Rarity</TableHead>
              <TableHead>Max</TableHead>
              <TableHead>Grow (s)</TableHead>
              <TableHead>Collision</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.items.map((item) => <ItemRow key={item.id} item={item} />) ?? null}
            {!loading && data?.items.length === 0 && (
              <TableRow>
                <TableCell colSpan={7} className="py-12 text-center text-muted-foreground">
                  No items found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </ScrollArea>

      <div className="shrink-0 flex items-center justify-between px-6 py-2 border-t border-border bg-card">
        <Button
          size="sm"
          variant="outline"
          className="h-7 text-xs gap-1"
          disabled={!data || page <= 1}
          onClick={() => setPage((p) => p - 1)}
        >
          <ChevronLeft className="w-3 h-3" />
          Prev
        </Button>
        <span className="text-xs text-muted-foreground">
          {data ? `Page ${page} / ${totalPages || 1}` : '—'}
        </span>
        <Button
          size="sm"
          variant="outline"
          className="h-7 text-xs gap-1"
          disabled={!data || page >= totalPages}
          onClick={() => setPage((p) => p + 1)}
        >
          Next
          <ChevronRight className="w-3 h-3" />
        </Button>
      </div>
    </div>
  )
}

function ItemRow({ item }: { item: ItemRecord }) {
  return (
    <TableRow>
      <TableCell className="font-mono text-muted-foreground">{item.id}</TableCell>
      <TableCell className="font-medium">{item.name}</TableCell>
      <TableCell className="text-muted-foreground">{item.action_type}</TableCell>
      <TableCell className="text-muted-foreground">{item.rarity}</TableCell>
      <TableCell className="text-muted-foreground">{item.max_item}</TableCell>
      <TableCell className="text-muted-foreground">{item.grow_time || '—'}</TableCell>
      <TableCell className="text-muted-foreground">{item.collision_type}</TableCell>
    </TableRow>
  )
}
