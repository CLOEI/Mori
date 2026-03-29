import { useState } from 'react'
import { CheckCircle2, XCircle, Loader2, ShieldCheck } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table'
import { api, type ProxyTestResult } from '@/lib/api'
import { cn } from '@/lib/utils'

interface ParsedProxy {
  host: string
  port: number
  username?: string
  password?: string
}

function parseProxyLine(line: string): ParsedProxy | null {
  const parts = line.trim().split(':')
  if (parts.length < 2) return null
  const [host, rawPort, username, password] = parts
  const port = Number(rawPort)
  if (!host || !port) return null
  return { host, port, username: username || undefined, password: password || undefined }
}

type EntryStatus = 'idle' | 'testing' | 'done'

interface ProxyEntry {
  raw: string
  status: EntryStatus
  result?: ProxyTestResult
  parseError?: string
}

export function ProxyPage() {
  const [input, setInput] = useState('')
  const [entries, setEntries] = useState<ProxyEntry[]>([])
  const [running, setRunning] = useState(false)

  function updateEntry(i: number, patch: Partial<ProxyEntry>) {
    setEntries(prev => prev.map((e, j) => (j === i ? { ...e, ...patch } : e)))
  }

  async function handleTest() {
    const lines = input.split('\n').map(l => l.trim()).filter(Boolean)
    if (!lines.length) return

    setEntries(lines.map(raw => ({ raw, status: 'idle' })))
    setRunning(true)

    await Promise.allSettled(
      lines.map(async (raw, i) => {
        const parsed = parseProxyLine(raw)
        if (!parsed) {
          updateEntry(i, { status: 'done', parseError: 'invalid format' })
          return
        }
        updateEntry(i, { status: 'testing' })
        try {
          const result = await api.testProxy({
            proxy_host: parsed.host,
            proxy_port: parsed.port,
            proxy_username: parsed.username,
            proxy_password: parsed.password,
          })
          updateEntry(i, { status: 'done', result })
        } catch (err) {
          updateEntry(i, {
            status: 'done',
            parseError: err instanceof Error ? err.message : String(err),
          })
        }
      })
    )

    setRunning(false)
  }

  const total = entries.length
  const done = entries.filter(e => e.status === 'done').length
  const ok = entries.filter(
    e => e.result?.socks5.ok && e.result?.server_data.ok && e.result?.enet.ok
  ).length

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="shrink-0 flex items-center gap-2 px-4 py-2.5 border-b border-border bg-card">
        <ShieldCheck className="w-3.5 h-3.5 text-muted-foreground" />
        <span className="text-xs font-medium text-foreground">Proxy Tester</span>
        {total > 0 && (
          <span className="ml-auto text-xs text-muted-foreground">
            {done}/{total}&nbsp;tested&nbsp;·&nbsp;
            <span className="text-green-500">{ok} ok</span>
            &nbsp;·&nbsp;
            <span className="text-destructive">{done - ok} failed</span>
          </span>
        )}
      </div>

      <div className="flex-1 flex overflow-hidden">
        {/* ── Input panel ── */}
        <div className="w-56 shrink-0 flex flex-col gap-2 p-3 border-r border-border">
          <label className="text-xs text-muted-foreground leading-tight">
            One proxy per line
            <span className="block font-mono text-[10px] mt-0.5 text-muted-foreground/60">
              host:port:user:pass
            </span>
          </label>
          <Textarea
            className="flex-1 font-mono text-xs placeholder:text-muted-foreground/50 min-h-0"
            placeholder={"103.160.95.181:1080:user:pass\n45.32.10.5:1080"}
            value={input}
            onChange={e => setInput(e.target.value)}
            spellCheck={false}
          />
          <Button
            size="sm"
            className="h-8 text-xs shrink-0"
            disabled={!input.trim() || running}
            onClick={handleTest}
          >
            {running ? (
              <>
                <Loader2 className="w-3 h-3 animate-spin mr-1.5" />
                Testing…
              </>
            ) : (
              'Test All'
            )}
          </Button>
        </div>

        {/* ── Results panel ── */}
        {entries.length === 0 ? (
          <div className="flex-1 flex items-center justify-center text-xs text-muted-foreground">
            paste proxies on the left and click Test All
          </div>
        ) : (
          <ScrollArea className="flex-1">
            <Table className="text-xs">
              <TableHeader className="sticky top-0 z-10 bg-card">
                <TableRow>
                  <TableHead className="w-full">Proxy</TableHead>
                  <TableHead className="text-center">SOCKS5</TableHead>
                  <TableHead className="text-center">server_data</TableHead>
                  <TableHead className="text-center">ENet UDP</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {entries.map((entry, i) => (
                  <ProxyRow key={i} entry={entry} />
                ))}
              </TableBody>
            </Table>
          </ScrollArea>
        )}
      </div>
    </div>
  )
}

function CheckIcon({
  check,
  testing,
}: {
  check?: { ok: boolean; error?: string }
  testing: boolean
}) {
  if (testing) return <Loader2 className="w-3.5 h-3.5 animate-spin text-muted-foreground mx-auto" />
  if (!check) return <span className="block w-2 h-2 rounded-full bg-muted mx-auto" />
  return check.ok ? (
    <CheckCircle2 className="w-3.5 h-3.5 text-green-500 mx-auto" />
  ) : (
    <XCircle className="w-3.5 h-3.5 text-destructive mx-auto" />
  )
}

function ProxyRow({ entry }: { entry: ProxyEntry }) {
  const testing = entry.status === 'testing'
  const display = entry.raw.split(':').slice(0, 2).join(':')

  const allOk = entry.result?.socks5.ok && entry.result?.server_data.ok && entry.result?.enet.ok

  return (
    <TableRow>
      <TableCell>
        <span
          className={cn(
            'font-mono',
            entry.status === 'done' && allOk && 'text-green-500',
            entry.status === 'done' && !allOk && 'text-foreground',
            entry.status !== 'done' && 'text-muted-foreground',
          )}
        >
          {display}
        </span>
        {entry.status === 'idle' && (
          <span className="ml-2 text-muted-foreground/60">waiting…</span>
        )}
        {entry.parseError && (
          <span className="ml-2 text-destructive/80 text-[11px]">{entry.parseError}</span>
        )}
        {entry.result?.server_data.detail && (
          <span className="ml-2 font-mono text-[10px] text-muted-foreground">
            → {entry.result.server_data.detail}
          </span>
        )}
      </TableCell>
      <TableCell className="text-center">
        <CheckIcon check={entry.result?.socks5} testing={testing} />
      </TableCell>
      <TableCell className="text-center">
        <CheckIcon check={entry.result?.server_data} testing={testing} />
      </TableCell>
      <TableCell className="text-center">
        <CheckIcon check={entry.result?.enet} testing={testing} />
      </TableCell>
    </TableRow>
  )
}
