import { useState, useRef, useEffect } from 'react'
import { X, Gem, Wifi, ChevronUp, ChevronDown, ChevronLeft, ChevronRight } from 'lucide-react'
import { useSetAtom, useAtomValue } from 'jotai'
import { selectedBotIdAtom, botsAtom, itemNamesAtom, type LiveBot } from '@/lib/store'
import { api } from '@/lib/api'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table'
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
  too_many_logins: 'bg-purple-500',
  update_required: 'bg-gray-500',
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
          inventory_slots: s.inventory_slots,
          gems: s.gems,
          console: s.console,
          ping_ms: s.ping_ms,
          delays: s.delays,
          track_info: s.track_info,
          auto_collect: s.auto_collect,
          collect_radius_tiles: s.collect_radius_tiles,
          collect_blacklist: s.collect_blacklist,
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

        <TabsContent value="overview" className="flex-1 overflow-hidden m-0">
          <ScrollArea className="h-full p-4">
            <OverviewTab bot={bot} />
          </ScrollArea>
        </TabsContent>

        <TabsContent value="console" className="flex-1 overflow-hidden m-0 p-4 flex flex-col">
          <ConsoleTab lines={bot.console} />
        </TabsContent>

        <TabsContent value="script" className="flex-1 overflow-hidden m-0 p-4 flex flex-col gap-3">
          <ScriptTab botId={bot.id} />
        </TabsContent>

        <TabsContent value="config" className="flex-1 overflow-auto m-0 p-4">
          <ConfigTab botId={bot.id} delays={bot.delays} autoCollect={bot.auto_collect} />
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
      {/* Left column: minimap + tiles + players */}
      <div className="flex flex-col gap-4">
        <Section label="Minimap" hint="click to walk · scroll to zoom">
          <Minimap bot={bot} />
        </Section>

        <Section label="Tiles in World">
          <TilesTable tiles={bot.tiles} itemLabel={itemLabel} />
        </Section>

        <Section label="Players in World">
          <DataTable
            columns={['Name', 'Net ID', 'Country']}
            rows={players.map((p) => [p.name, String(p.net_id), p.country])}
            empty="No players"
          />
        </Section>
      </div>

      {/* Right column: account + inventory + objects */}
      <div className="flex flex-col gap-4">
        <Section label="Account">
          <div className="rounded border border-border bg-background p-2 grid grid-cols-2 gap-x-4 gap-y-1 text-[11px]">
            {[
              ['Level', bot.track_info?.level],
              ['Grow ID', bot.track_info?.grow_id],
              ['Awesomeness', bot.track_info?.awesomeness],
              ['Playtime', bot.track_info ? `${Math.floor(bot.track_info.global_playtime / 3600)}h` : null],
              ['Install Date', bot.track_info ? new Date(bot.track_info.install_date * 1000).toLocaleDateString() : null],
            ].map(([label, val]) => (
              <>
                <span key={`${label}-k`} className="text-muted-foreground">{label}</span>
                <span key={`${label}-v`} className="text-foreground">{val ?? '—'}</span>
              </>
            ))}
          </div>
        </Section>

        <AutoCollectRangePanel
          botId={bot.id}
          collectRadiusTiles={bot.collect_radius_tiles}
          collectBlacklist={bot.collect_blacklist}
        />

        <Section label="Movement">
          <DPad botId={bot.id} />
        </Section>

        <Section
            label="Inventory"
            hint={bot.inventory_slots > 0 ? `${bot.inventory.length} / ${bot.inventory_slots} slots` : undefined}
          >
          <InventoryTable botId={bot.id} inventory={bot.inventory} itemLabel={itemLabel} />
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
    </div>
  )
}

// ── Overview: auto-collect range + blacklist ────────────────────────────────

/** 9×9 so Chebyshev distance from center runs 0..4 → clickable radius 1..5 (was 5×5 → max 3). */
const COLLECT_GRID = 9
const COLLECT_MID = (COLLECT_GRID - 1) / 2

function collectChebyshev(row: number, col: number) {
  return Math.max(Math.abs(row - COLLECT_MID), Math.abs(col - COLLECT_MID))
}

function collectRadiusFromCell(row: number, col: number) {
  return Math.min(5, Math.max(1, collectChebyshev(row, col) + 1))
}

function cellInsideCollectRadius(row: number, col: number, radiusTiles: number) {
  return collectChebyshev(row, col) < radiusTiles
}

function AutoCollectRangePanel({
  botId,
  collectRadiusTiles,
  collectBlacklist,
}: {
  botId: number
  collectRadiusTiles: number
  collectBlacklist: number[]
}) {
  const [collectRadius, setCollectRadius] = useState(collectRadiusTiles)
  const [blacklist, setBlacklist] = useState<number[]>(() => [...collectBlacklist])
  const [blacklistInput, setBlacklistInput] = useState('')
  const setBots = useSetAtom(botsAtom)
  const itemNames = useAtomValue(itemNamesAtom)

  useEffect(() => {
    setCollectRadius(collectRadiusTiles)
    setBlacklist([...collectBlacklist].sort((a, b) => a - b))
  }, [collectRadiusTiles, collectBlacklist.join(',')])

  const itemLabel = (id: number) => {
    const name = itemNames[String(id)]
    return name ? `${name} (${id})` : String(id)
  }

  async function sendCollectConfig(nextRadius: number, nextBlacklist: number[]) {
    const prevRadius = collectRadius
    const prevBlacklist = blacklist
    setCollectRadius(nextRadius)
    setBlacklist(nextBlacklist)
    setBots((m) => {
      const b = m.get(botId)
      if (!b) return m
      return new Map(m).set(botId, {
        ...b,
        collect_radius_tiles: nextRadius,
        collect_blacklist: [...nextBlacklist].sort((a, b) => a - b),
      })
    })
    try {
      await api.sendCmd(botId, {
        type: 'set_collect_config',
        radius_tiles: nextRadius,
        blacklist: nextBlacklist,
      })
    } catch {
      setCollectRadius(prevRadius)
      setBlacklist(prevBlacklist)
      setBots((m) => {
        const b = m.get(botId)
        if (!b) return m
        return new Map(m).set(botId, {
          ...b,
          collect_radius_tiles: prevRadius,
          collect_blacklist: [...prevBlacklist].sort((a, b) => a - b),
        })
      })
    }
  }

  function onCollectCellClick(row: number, col: number) {
    const next = collectRadiusFromCell(row, col)
    void sendCollectConfig(next, blacklist)
  }

  function addBlacklistId() {
    const id = parseInt(blacklistInput.trim(), 10)
    if (!Number.isFinite(id) || id < 0 || id > 65535) return
    if (blacklist.includes(id)) return
    const next = [...blacklist, id].sort((a, b) => a - b)
    setBlacklistInput('')
    void sendCollectConfig(collectRadius, next)
  }

  function removeBlacklistId(id: number) {
    const next = blacklist.filter((x) => x !== id)
    void sendCollectConfig(collectRadius, next)
  }

  return (
    <>
      <Section
        label="Auto-collect range"
        hint="square in tiles"
      >
        <div className="flex flex-col items-center gap-2 w-full">
          <div
            className="grid gap-0.5 w-fit shrink-0 p-1 rounded border border-border bg-muted/30"
            style={{ gridTemplateColumns: `repeat(${COLLECT_GRID}, minmax(0, 1fr))` }}
          >
            {Array.from({ length: COLLECT_GRID * COLLECT_GRID }, (_, i) => {
              const row = Math.floor(i / COLLECT_GRID)
              const col = i % COLLECT_GRID
              const inRange = cellInsideCollectRadius(row, col, collectRadius)
              const isCenter = row === COLLECT_MID && col === COLLECT_MID
              return (
                <button
                  key={i}
                  type="button"
                  title={`Set radius ${collectRadiusFromCell(row, col)}`}
                  onClick={() => onCollectCellClick(row, col)}
                  className={cn(
                    'w-6 h-6 rounded-sm border text-[9px] font-medium transition-colors',
                    inRange
                      ? 'bg-primary/35 border-primary/50 text-foreground'
                      : 'bg-background/80 border-border text-muted-foreground',
                    isCenter && 'ring-1 ring-primary ring-inset',
                  )}
                >
                  {isCenter ? '●' : ''}
                </button>
              )
            })}
          </div>
          <p className="text-xs text-muted-foreground text-center">
            Radius: <span className="text-foreground font-medium tabular-nums">{collectRadius}</span> tile{collectRadius === 1 ? '' : 's'}
          </p>
        </div>
      </Section>
      <Section label="Do not auto-collect" hint="item IDs">
        <div className="flex gap-2">
          <Input
            type="number"
            min={0}
            max={65535}
            placeholder="Item ID"
            value={blacklistInput}
            onChange={(e) => setBlacklistInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && addBlacklistId()}
            className="h-7 text-xs"
          />
          <Button size="sm" variant="secondary" className="text-xs shrink-0 h-7" type="button" onClick={addBlacklistId}>
            Add
          </Button>
        </div>
        <ul className="max-h-32 overflow-y-auto rounded border border-border bg-background divide-y divide-border text-xs">
          {blacklist.length === 0 ? (
            <li className="px-2 py-2 text-muted-foreground text-center">No blacklisted items</li>
          ) : (
            blacklist.map((id) => (
              <li key={id} className="flex items-center justify-between gap-2 px-2 py-1.5">
                <span className="truncate">{itemLabel(id)}</span>
                <button
                  type="button"
                  onClick={() => removeBlacklistId(id)}
                  className="shrink-0 text-muted-foreground hover:text-destructive text-[10px] uppercase tracking-wide"
                >
                  Remove
                </button>
              </li>
            ))
          )}
        </ul>
      </Section>
    </>
  )
}

// ── Console tab ─────────────────────────────────────────────────────────────

function ConsoleTab({ lines }: { lines: string[] }) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [lines])

  return (
    <ScrollArea className="flex-1 min-h-0 rounded border border-border bg-background p-3 font-mono text-[12px] leading-relaxed">
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
  autoCollect,
}: {
  botId: number
  delays: { place_ms: number; walk_ms: number; twofa_secs: number; server_overload_secs: number; too_many_logins_secs: number }
  autoCollect: boolean
}) {
  const [placeMs, setPlaceMs] = useState(String(delays.place_ms))
  const [walkMs, setWalkMs] = useState(String(delays.walk_ms))
  const [twofaSecs, setTwofaSecs] = useState(String(delays.twofa_secs))
  const [serverOverloadSecs, setServerOverloadSecs] = useState(String(delays.server_overload_secs))
  const [tooManyLoginsSecs, setTooManyLoginsSecs] = useState(String(delays.too_many_logins_secs))
  const [status, setStatus] = useState('')
  const setBots = useSetAtom(botsAtom)

  // sync when bot prop changes
  useEffect(() => {
    setPlaceMs(String(delays.place_ms))
    setWalkMs(String(delays.walk_ms))
    setTwofaSecs(String(delays.twofa_secs))
    setServerOverloadSecs(String(delays.server_overload_secs))
    setTooManyLoginsSecs(String(delays.too_many_logins_secs))
  }, [delays.place_ms, delays.walk_ms, delays.twofa_secs, delays.server_overload_secs, delays.too_many_logins_secs])

  async function save() {
    try {
      await api.sendCmd(botId, {
        type: 'set_delays',
        place_ms: parseInt(placeMs, 10),
        walk_ms: parseInt(walkMs, 10),
        twofa_secs: parseInt(twofaSecs, 10),
        server_overload_secs: parseInt(serverOverloadSecs, 10),
        too_many_logins_secs: parseInt(tooManyLoginsSecs, 10),
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
      <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        Pending Retry Delays (s)
      </p>
      <div className="flex flex-col gap-3">
        <label className="flex flex-col gap-1">
          <span className="text-xs text-muted-foreground">2FA / Account Protection (s)</span>
          <Input
            type="number"
            min={1}
            step={1}
            value={twofaSecs}
            onChange={(e) => setTwofaSecs(e.target.value)}
            className="h-7 text-xs"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs text-muted-foreground">Server Overloaded (s)</span>
          <Input
            type="number"
            min={1}
            step={1}
            value={serverOverloadSecs}
            onChange={(e) => setServerOverloadSecs(e.target.value)}
            className="h-7 text-xs"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-xs text-muted-foreground">Too Many Logins (s)</span>
          <Input
            type="number"
            min={1}
            step={1}
            value={tooManyLoginsSecs}
            onChange={(e) => setTooManyLoginsSecs(e.target.value)}
            className="h-7 text-xs"
          />
        </label>
      </div>
      <div className="flex items-center gap-2">
        <Button size="sm" className="text-xs" onClick={save}>Save</Button>
        {status && <span className="text-xs text-muted-foreground">{status}</span>}
      </div>
      <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        Behaviour
      </p>
      <label className="flex items-center gap-2 cursor-pointer">
        <Switch
          size="sm"
          checked={autoCollect}
          onCheckedChange={(enabled) => {
            setBots((m) => {
              const bot = m.get(botId)
              if (!bot) return m
              return new Map(m).set(botId, { ...bot, auto_collect: enabled })
            })
            api.sendCmd(botId, { type: 'set_auto_collect', enabled }).catch(() => {
              setBots((m) => {
                const bot = m.get(botId)
                if (!bot) return m
                return new Map(m).set(botId, { ...bot, auto_collect: !enabled })
              })
            })
          }}
        />
        <span className="text-xs text-muted-foreground">Auto-collect dropped items</span>
      </label>
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
    <ScrollArea className={cn('rounded border border-border', maxH)}>
      <Table className="text-xs">
        <TableHeader className="sticky top-0 bg-card">
          <TableRow className="border-b border-border">
            {columns.map((c) => (
              <TableHead key={c}>{c}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.length === 0 ? (
            <TableRow>
              <TableCell colSpan={columns.length} className="py-2 text-center text-muted-foreground">
                {empty}
              </TableCell>
            </TableRow>
          ) : (
            rows.map((row, i) => (
              <TableRow key={i}>
                {row.map((cell, j) => (
                  <TableCell key={j}>{cell}</TableCell>
                ))}
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </ScrollArea>
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
  async function cmd(type: string, item_id: number, count?: number) {
    await api.sendCmd(botId, { type, item_id, ...(count !== undefined ? { count } : {}) } as never)
  }

  return (
    <ScrollArea className="max-h-72 rounded border border-border">
      <Table className="text-xs">
        <TableHeader className="sticky top-0 bg-card">
          <TableRow className="border-b border-border">
            <TableHead>Item</TableHead>
            <TableHead>Amt</TableHead>
            <TableHead>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {inventory.length === 0 ? (
            <TableRow>
              <TableCell colSpan={3} className="py-2 text-center text-muted-foreground">
                Empty
              </TableCell>
            </TableRow>
          ) : [...inventory].sort((a, b) => b.amount - a.amount).map((i) => (
            <TableRow key={i.item_id}>
              <TableCell>
                {i.is_active && <span className="text-primary mr-1">●</span>}
                {itemLabel(i.item_id)}
              </TableCell>
              <TableCell>{i.amount}</TableCell>
              <TableCell className="px-2 py-1">
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
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </ScrollArea>
  )
}

function TilesTable({
  tiles,
  itemLabel,
}: {
  tiles: { fg: number; bg: number; flags: number; tile_type: unknown }[]
  itemLabel: (id: number) => string
}) {
  const counts = new Map<number, number>()
  for (const t of tiles) {
    if (t.fg !== 0) counts.set(t.fg, (counts.get(t.fg) ?? 0) + 1)
  }
  const rows = [...counts.entries()]
    .sort((a, b) => b[1] - a[1])
    .map(([id, count]) => [itemLabel(id), count.toLocaleString()])
  return <DataTable columns={['Tile', 'Count']} rows={rows} empty="No tiles" />
}

function DPad({ botId }: { botId: number }) {
  const move = (x: number, y: number) => api.sendCmd(botId, { type: 'move', x, y }).catch(() => {})

  return (
    <div className="grid grid-cols-3 gap-1 w-fit mx-auto">
      <div />
      <button onClick={() => move(0, -1)} className="flex items-center justify-center w-8 h-8 rounded bg-secondary hover:bg-secondary/80 text-secondary-foreground transition-colors">
        <ChevronUp className="w-4 h-4" />
      </button>
      <div />
      <button onClick={() => move(-1, 0)} className="flex items-center justify-center w-8 h-8 rounded bg-secondary hover:bg-secondary/80 text-secondary-foreground transition-colors">
        <ChevronLeft className="w-4 h-4" />
      </button>
      <button onClick={() => move(0, 1)} className="flex items-center justify-center w-8 h-8 rounded bg-secondary hover:bg-secondary/80 text-secondary-foreground transition-colors">
        <ChevronDown className="w-4 h-4" />
      </button>
      <button onClick={() => move(1, 0)} className="flex items-center justify-center w-8 h-8 rounded bg-secondary hover:bg-secondary/80 text-secondary-foreground transition-colors">
        <ChevronRight className="w-4 h-4" />
      </button>
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
