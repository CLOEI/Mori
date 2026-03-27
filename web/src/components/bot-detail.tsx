import { useState, useRef, useEffect } from 'react'
import { X, Gem, Wifi } from 'lucide-react'
import { useSetAtom, useAtomValue } from 'jotai'
import { selectedBotIdAtom, botsAtom, itemNamesAtom, type LiveBot } from '@/lib/store'
import { api } from '@/lib/api'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { parseGTColors } from '@/lib/gt-colors'
import { Minimap } from '@/components/minimap'
import { cn } from '@/lib/utils'
import type { BotStatus } from '@/lib/api'

const STATUS_DOT: Record<BotStatus, string> = {
  connecting: 'bg-yellow-500',
  connected: 'bg-blue-500',
  in_world: 'bg-emerald-500',
  two_factor_auth: 'bg-orange-500',
  server_overloaded: 'bg-red-500',
}

export function BotDetail({ bot }: { bot: LiveBot }) {
  const setSelectedId = useSetAtom(selectedBotIdAtom)
  const setBots = useSetAtom(botsAtom)

  // Seed full state from REST on mount / bot change
  useEffect(() => {
    api.getBotState(bot.id).then((s) => {
      setBots((m) => {
        const existing = m.get(bot.id)
        if (!existing) return m
        const players = new Map(
          s.players.map((p) => [p.net_id, p])
        )
        const tiles = s.tiles.map((t) => ({
          fg: t.fg_item_id,
          bg: t.bg_item_id,
          flags: t.flags,
          tile_type: t.tile_type,
        }))
        return new Map(m).set(bot.id, {
          ...existing,
          status: s.status,
          world_name: s.world_name,
          pos_x: s.pos_x,
          pos_y: s.pos_y,
          world_width: s.world_width,
          world_height: s.world_height,
          tiles,
          players,
          objects: s.objects,
          inventory: s.inventory,
          gems: s.gems,
          console: s.console,
          ping_ms: s.ping_ms,
          delays: s.delays,
          track_info: s.track_info,
        })
      })
    }).catch(() => {})
  }, [bot.id, setBots])

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Header bar */}
      <div className="shrink-0 flex items-center gap-2 px-4 h-10 bg-card border-b border-border">
        <span className={cn('w-2 h-2 rounded-full shrink-0', STATUS_DOT[bot.status])} />
        <span className="font-semibold text-xs">{bot.username}</span>
        {bot.world_name && (
          <>
            <span className="text-border text-xs">|</span>
            <span className="text-xs text-muted-foreground">{bot.world_name}</span>
            <span className="text-xs text-muted-foreground/50">
              ({bot.pos_x.toFixed(1)}, {bot.pos_y.toFixed(1)})
            </span>
          </>
        )}
        <div className="flex-1" />
        {bot.gems > 0 && (
          <span className="flex items-center gap-1 text-xs text-emerald-500 font-medium">
            <Gem className="w-3 h-3" />
            {bot.gems.toLocaleString()}
          </span>
        )}
        {bot.ping_ms > 0 && (
          <span className="flex items-center gap-1 text-xs text-muted-foreground">
            <Wifi className="w-3 h-3" />
            {bot.ping_ms}ms
          </span>
        )}
        <button
          onClick={() => setSelectedId(null)}
          className="ml-1 w-5 h-5 flex items-center justify-center rounded hover:bg-muted text-muted-foreground hover:text-foreground text-xs transition-colors"
        >
          <X className="w-3 h-3" />
        </button>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="overview" className="flex-1 flex flex-col overflow-hidden">
        <TabsList className="shrink-0 w-full justify-start rounded-none border-b border-border bg-card/40 h-9 px-2 gap-0">
          {(['overview', 'console', 'script', 'config'] as const).map((t) => (
            <TabsTrigger
              key={t}
              value={t}
              className="rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none px-4 h-full text-xs capitalize"
            >
              {t}
            </TabsTrigger>
          ))}
        </TabsList>

        <TabsContent value="overview" className="flex-1 overflow-auto m-0 p-4">
          <OverviewTab bot={bot} />
        </TabsContent>

        <TabsContent value="console" className="flex-1 overflow-hidden m-0 p-4 flex flex-col">
          <ConsoleTab lines={bot.console} />
        </TabsContent>

        <TabsContent value="script" className="flex-1 overflow-hidden m-0 p-4 flex flex-col gap-3">
          <ScriptTab botId={bot.id} />
        </TabsContent>

        <TabsContent value="config" className="flex-1 overflow-auto m-0 p-4">
          <ConfigTab botId={bot.id} delays={bot.delays} />
        </TabsContent>
      </Tabs>
    </div>
  )
}

// ── Overview tab ────────────────────────────────────────────────────────────

function OverviewTab({ bot }: { bot: LiveBot }) {
  const players = [...bot.players.values()]
  const itemNames = useAtomValue(itemNamesAtom)
  const itemLabel = (id: number) => {
    const name = itemNames[String(id)]
    return name ? `${name} (${id})` : String(id)
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
      {/* Left column: minimap + players + objects */}
      <div className="flex flex-col gap-4">
        <Section label="Minimap" hint="click to walk · scroll to zoom">
          <Minimap bot={bot} />
        </Section>

        <Section label="Players in World">
          <DataTable
            columns={['Name', 'Net ID', 'Country']}
            rows={players.map((p) => [p.name, String(p.net_id), p.country])}
            empty="No players"
          />
        </Section>

        <Section label="Floating Objects">
          <DataTable
            columns={['Item', 'Amt', 'Pos', 'UID']}
            rows={bot.objects.map((o) => [
              itemLabel(o.item_id),
              String(o.count),
              `${o.x.toFixed(1)},${o.y.toFixed(1)}`,
              String(o.uid),
            ])}
            empty="No objects"
          />
        </Section>
      </div>

      {/* Right column: account + inventory */}
      <div className="flex flex-col gap-4">
        <Section label="Account">
          <div className="rounded border border-border bg-background p-2 grid grid-cols-2 gap-x-4 gap-y-1 text-[11px]">
            {[
              ['Level', bot.track_info?.level],
              ['Grow ID', bot.track_info?.grow_id],
              ['Awesomeness', bot.track_info?.awesomeness],
              ['Playtime', bot.track_info ? `${Math.floor(bot.track_info.global_playtime / 3600)}h` : null],
            ].map(([label, val]) => (
              <>
                <span key={`${label}-k`} className="text-muted-foreground">{label}</span>
                <span key={`${label}-v`} className="text-foreground">{val ?? '—'}</span>
              </>
            ))}
          </div>
        </Section>

        <Section label="Inventory">
          <InventoryTable botId={bot.id} inventory={bot.inventory} itemLabel={itemLabel} />
        </Section>
      </div>
    </div>
  )
}

// ── Console tab ─────────────────────────────────────────────────────────────

function ConsoleTab({ lines }: { lines: string[] }) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [lines])

  return (
    <ScrollArea className="flex-1 rounded border border-border bg-background p-3 font-mono text-[12px] leading-relaxed">
      {lines.length === 0 ? (
        <span className="text-muted-foreground">No output yet.</span>
      ) : (
        lines.map((line, i) => (
          <div
            key={i}
            className="whitespace-pre-wrap break-all"
            dangerouslySetInnerHTML={{ __html: parseGTColors(line) }}
          />
        ))
      )}
      <div ref={bottomRef} />
    </ScrollArea>
  )
}

// ── Script tab ──────────────────────────────────────────────────────────────

function ScriptTab({ botId }: { botId: number }) {
  const [script, setScript] = useState('')
  const [status, setStatus] = useState('')

  async function run() {
    try {
      await api.sendCmd(botId, { type: 'run_script', content: script })
      setStatus('Running…')
    } catch {
      setStatus('Error')
    }
  }

  async function stop() {
    try {
      await api.sendCmd(botId, { type: 'stop_script' })
      setStatus('Stopped')
    } catch {
      setStatus('Error')
    }
  }

  return (
    <>
      <textarea
        value={script}
        onChange={(e) => setScript(e.target.value)}
        spellCheck={false}
        className="flex-1 font-mono text-[13px] bg-background border border-border rounded p-3 text-foreground resize-none outline-none leading-relaxed focus:border-ring transition-colors"
        placeholder={"-- Lua script\nbot:warp(\"START\")\nsleep(500)"}
      />
      <div className="shrink-0 flex items-center gap-2">
        <Button size="sm" variant="default" className="text-xs" onClick={run}>Run</Button>
        <Button size="sm" variant="destructive" className="text-xs" onClick={stop}>Stop</Button>
        {status && <span className="text-xs text-muted-foreground">{status}</span>}
      </div>
    </>
  )
}

// ── Config tab ──────────────────────────────────────────────────────────────

function ConfigTab({
  botId,
  delays,
}: {
  botId: number
  delays: { place_ms: number; walk_ms: number }
}) {
  const [placeMs, setPlaceMs] = useState(String(delays.place_ms))
  const [walkMs, setWalkMs] = useState(String(delays.walk_ms))
  const [status, setStatus] = useState('')

  // sync when bot prop changes
  useEffect(() => {
    setPlaceMs(String(delays.place_ms))
    setWalkMs(String(delays.walk_ms))
  }, [delays.place_ms, delays.walk_ms])

  async function save() {
    try {
      await api.sendCmd(botId, {
        type: 'set_delays',
        place_ms: parseInt(placeMs, 10),
        walk_ms: parseInt(walkMs, 10),
      })
      setStatus('Saved')
      setTimeout(() => setStatus(''), 2000)
    } catch {
      setStatus('Error')
    }
  }

  return (
    <div className="max-w-xs flex flex-col gap-4">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        Action Delays (ms)
      </p>
      <div className="flex flex-col gap-3">
        <label className="flex flex-col gap-1">
          <span className="text-xs text-muted-foreground">Place / Punch (ms)</span>
          <Input
            type="number"
            min={0}
            step={50}
            value={placeMs}
            onChange={(e) => setPlaceMs(e.target.value)}
            className="h-7 text-xs"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs text-muted-foreground">Walk / Pathfind (ms)</span>
          <Input
            type="number"
            min={0}
            step={50}
            value={walkMs}
            onChange={(e) => setWalkMs(e.target.value)}
            className="h-7 text-xs"
          />
        </label>
      </div>
      <div className="flex items-center gap-2">
        <Button size="sm" className="text-xs" onClick={save}>Save</Button>
        {status && <span className="text-xs text-muted-foreground">{status}</span>}
      </div>
    </div>
  )
}

// ── Shared helpers ──────────────────────────────────────────────────────────

function Section({
  label,
  hint,
  children,
}: {
  label: string
  hint?: string
  children: React.ReactNode
}) {
  return (
    <div className="flex flex-col gap-2">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        {label}
        {hint && <span className="font-normal normal-case tracking-normal text-muted-foreground/50 ml-1">— {hint}</span>}
      </p>
      {children}
    </div>
  )
}

function DataTable({
  columns,
  rows,
  empty,
  maxH = 'max-h-40',
}: {
  columns: string[]
  rows: string[][]
  empty: string
  maxH?: string
}) {
  return (
    <div className={cn('overflow-y-auto rounded border border-border', maxH)}>
      <table className="w-full text-xs">
        <thead>
          <tr className="bg-card text-[10px] uppercase tracking-wide text-muted-foreground sticky top-0">
            {columns.map((c) => (
              <th key={c} className="px-3 py-1.5 text-left font-semibold">{c}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className="px-3 py-2 text-center text-muted-foreground">
                {empty}
              </td>
            </tr>
          ) : (
            rows.map((row, i) => (
              <tr key={i} className="border-t border-border/50 hover:bg-muted/50 transition-colors">
                {row.map((cell, j) => (
                  <td key={j} className="px-3 py-1.5 text-foreground/80">{cell}</td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  )
}

function InventoryTable({
  botId,
  inventory,
  itemLabel,
}: {
  botId: number
  inventory: import('@/lib/api').InventoryItem[]
  itemLabel: (id: number) => string
}) {
  if (inventory.length === 0) {
    return <p className="text-xs text-muted-foreground px-1">Empty</p>
  }

  async function cmd(type: string, item_id: number, count?: number) {
    await api.sendCmd(botId, { type, item_id, ...(count !== undefined ? { count } : {}) } as never)
  }

  return (
    <div className="overflow-y-auto max-h-72 rounded border border-border">
      <table className="w-full text-xs">
        <thead>
          <tr className="bg-card text-[10px] uppercase tracking-wide text-muted-foreground sticky top-0">
            <th className="px-3 py-1.5 text-left font-semibold">Item</th>
            <th className="px-3 py-1.5 text-left font-semibold">Amt</th>
            <th className="px-3 py-1.5 text-left font-semibold">Actions</th>
          </tr>
        </thead>
        <tbody>
          {[...inventory].sort((a, b) => b.amount - a.amount).map((i) => (
            <tr key={i.item_id} className="border-t border-border/50 hover:bg-muted/50 transition-colors">
              <td className="px-3 py-1.5 text-foreground/80">
                {i.is_active && <span className="text-primary mr-1">●</span>}
                {itemLabel(i.item_id)}
              </td>
              <td className="px-3 py-1.5 text-foreground/80">{i.amount}</td>
              <td className="px-2 py-1">
                <div className="flex gap-1 flex-wrap">
                  {i.action_type === 20 && (
                    i.is_active
                      ? <Btn onClick={() => cmd('unwear', i.item_id)} variant="active">Unwear</Btn>
                      : <Btn onClick={() => cmd('wear', i.item_id)}>Wear</Btn>
                  )}
                  <Btn onClick={() => cmd('drop', i.item_id, 1)}>Drop</Btn>
                  <Btn onClick={() => cmd('drop', i.item_id, i.amount)}>Drop All</Btn>
                  <Btn onClick={() => cmd('trash', i.item_id, 1)} variant="danger">Trash</Btn>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function Btn({
  onClick,
  variant = 'default',
  children,
}: {
  onClick: () => void
  variant?: 'default' | 'active' | 'danger'
  children: React.ReactNode
}) {
  const cls = {
    default: 'bg-secondary hover:bg-secondary/80 text-secondary-foreground',
    active:  'bg-primary/20 hover:bg-primary/30 text-primary',
    danger:  'bg-destructive/20 hover:bg-destructive/30 text-destructive',
  }[variant]
  return (
    <button
      onClick={onClick}
      className={cn('px-1.5 py-0.5 rounded text-[10px] transition-colors', cls)}
    >
      {children}
    </button>
  )
}
