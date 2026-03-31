import { useState } from 'react'
import { Plus, Trash2, ChevronDown, ChevronRight, Loader2, X } from 'lucide-react'
import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import { botsAtom, selectedBotIdAtom } from '@/lib/store'
import { api, type BotStatus, type SpawnBotBody, type SpawnLtokenBody } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { cn } from '@/lib/utils'

const STATUS_COLOR: Record<BotStatus, string> = {
  connecting: 'bg-yellow-500',
  connected: 'bg-blue-500',
  in_game: 'bg-emerald-500',
  two_factor_auth: 'bg-orange-500',
  server_overloaded: 'bg-red-500',
  too_many_logins: 'bg-purple-500',
  update_required: 'bg-gray-500',
}

const STATUS_LABEL: Record<BotStatus, string> = {
  connecting: 'Connecting',
  connected: 'Connected',
  in_game: 'In Game',
  two_factor_auth: '2FA',
  server_overloaded: 'Overloaded',
  too_many_logins: 'Too Many Logins',
  update_required: 'Update Required',
}

export function BotSidebar({ onClose }: { onClose?: () => void }) {
  const [showForm, setShowForm] = useState(false)

  return (
    <aside className="w-52 h-full shrink-0 flex flex-col bg-card border-r border-border">
      {/* Add bot / close row */}
      <div className="shrink-0 p-2 border-b border-border flex gap-1.5">
        <Button
          size="sm"
          className="flex-1 text-xs gap-1.5"
          onClick={() => setShowForm((v) => !v)}
        >
          <Plus className="w-3 h-3" />
          Add Bot
        </Button>
        {onClose && (
          <Button size="sm" variant="ghost" className="px-2 md:hidden" onClick={onClose}>
            <X className="w-3 h-3" />
          </Button>
        )}
      </div>

      {/* Add bot form */}
      {showForm && (
        <div className="shrink-0 border-b border-border p-2">
          <AddBotForm onDone={() => setShowForm(false)} />
        </div>
      )}

      {/* Bot list */}
      <ScrollArea className="flex-1">
        <BotList />
      </ScrollArea>
    </aside>
  )
}

// ── Add Bot Form ────────────────────────────────────────────────────────────

function AddBotForm({ onDone }: { onDone: () => void }) {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [ltokenStr, setLtokenStr] = useState('')
  const [showProxy, setShowProxy] = useState(false)
  const [proxyStr, setProxyStr] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [method, setMethod] = useState<'legacy' | 'ltoken'>('legacy')

  function buildProxy(body: SpawnBotBody | SpawnLtokenBody) {
    if (showProxy && proxyStr) {
      const parts = proxyStr.split(':')
      if (parts.length >= 2) {
        body.proxy_host = parts[0]
        body.proxy_port = parseInt(parts[1], 10)
        if (parts[2]) body.proxy_username = parts[2]
        if (parts[3]) body.proxy_password = parts[3]
      }
    }
  }

  async function handleSubmit(e: React.SyntheticEvent<HTMLFormElement>) {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      if (method === 'legacy') {
        const body: SpawnBotBody = { username, password }
        buildProxy(body)
        await api.spawnBot(body)
      } else {
        const body: SpawnLtokenBody = { ltoken: ltokenStr }
        buildProxy(body)
        await api.spawnLtokenBot(body)
      }
      onDone()
    } catch {
      setError('Failed to spawn bot')
    } finally {
      setLoading(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-1.5">
      <Tabs value={method} onValueChange={(v) => setMethod(v as 'legacy' | 'ltoken')}>
        <TabsList className="w-full h-7">
          <TabsTrigger value="legacy" className="flex-1 text-[10px]">Legacy</TabsTrigger>
          <TabsTrigger value="ltoken" className="flex-1 text-[10px]">Ltoken</TabsTrigger>
        </TabsList>

        <TabsContent value="legacy" className="flex flex-col gap-1.5 mt-1.5">
          <Input
            placeholder="Username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className="h-7 text-xs"
            required={method === 'legacy'}
          />
          <Input
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="h-7 text-xs"
            required={method === 'legacy'}
          />
        </TabsContent>

        <TabsContent value="ltoken" className="mt-1.5">
          <Input
            placeholder="token|rid|mac|wk"
            value={ltokenStr}
            onChange={(e) => setLtokenStr(e.target.value)}
            className="h-7 text-xs font-mono"
            required={method === 'ltoken'}
          />
        </TabsContent>
      </Tabs>

      <button
        type="button"
        onClick={() => setShowProxy((v) => !v)}
        className="text-[10px] text-muted-foreground hover:text-foreground text-left flex items-center gap-1 transition-colors"
      >
        {showProxy ? <ChevronDown className="w-2.5 h-2.5" /> : <ChevronRight className="w-2.5 h-2.5" />}
        Proxy
      </button>
      {showProxy && (
        <Input
          placeholder="host:port:user:pass"
          value={proxyStr}
          onChange={(e) => setProxyStr(e.target.value)}
          className="h-7 text-xs font-mono"
        />
      )}

      <Button type="submit" size="sm" className="w-full text-xs" disabled={loading}>
        {loading ? <Loader2 className="w-3 h-3 animate-spin" /> : 'Spawn'}
      </Button>
      {error && <p className="text-[10px] text-destructive text-center">{error}</p>}
    </form>
  )
}

// ── Bot List ────────────────────────────────────────────────────────────────

function BotList() {
  const bots = useAtomValue(botsAtom)
  const [selectedId, setSelectedId] = useAtom(selectedBotIdAtom)

  if (bots.size === 0) {
    return (
      <p className="text-xs text-muted-foreground text-center py-6 px-2">
        No bots running.
      </p>
    )
  }

  return (
    <div className="p-1.5 space-y-1">
      {[...bots.values()].map((bot) => (
        <BotItem
          key={bot.id}
          bot={bot}
          selected={selectedId === bot.id}
          onSelect={() => setSelectedId(bot.id)}
        />
      ))}
    </div>
  )
}

function BotItem({
  bot,
  selected,
  onSelect,
}: {
  bot: { id: number; username: string; status: BotStatus; world_name: string; gems: number }
  selected: boolean
  onSelect: () => void
}) {
  const setBots = useSetAtom(botsAtom)
  const setSelectedId = useSetAtom(selectedBotIdAtom)

  async function handleDelete(e: React.MouseEvent) {
    e.stopPropagation()
    try {
      await api.deleteBot(bot.id)
      setBots((m) => { const n = new Map(m); n.delete(bot.id); return n })
      setSelectedId((id) => (id === bot.id ? null : id))
    } catch {
      // ignore
    }
  }

  return (
    <div
      onClick={onSelect}
      className={cn(
        'group flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer transition-colors',
        selected ? 'bg-accent text-accent-foreground' : 'hover:bg-muted',
      )}
    >
      <span className={cn('w-1.5 h-1.5 rounded-full shrink-0', STATUS_COLOR[bot.status])} />
      <div className="flex-1 min-w-0">
        <p className="text-xs font-medium truncate leading-tight">{bot.username}</p>
        {bot.world_name ? (
          <p className="text-[10px] text-muted-foreground truncate leading-tight">{bot.world_name}</p>
        ) : (
          <p className="text-[10px] text-muted-foreground leading-tight">{STATUS_LABEL[bot.status]}</p>
        )}
      </div>
      <button
        onClick={handleDelete}
        className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:text-destructive transition-all shrink-0"
      >
        <Trash2 className="w-3 h-3" />
      </button>
    </div>
  )
}
