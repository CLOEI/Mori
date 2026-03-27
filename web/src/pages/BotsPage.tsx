import { useState } from 'react'
import { useMoriStore, selectedBotAtom } from '@/lib/store'
import { useAtomValue } from 'jotai'
import { BotSidebar } from '@/components/bot-sidebar'
import { BotDetail } from '@/components/bot-detail'
import { Monitor, Menu } from 'lucide-react'
import { cn } from '@/lib/utils'

export function BotsPage() {
  useMoriStore()

  const selected = useAtomValue(selectedBotAtom)
  const [sidebarOpen, setSidebarOpen] = useState(false)

  return (
    <div className="h-full flex overflow-hidden relative">
      {sidebarOpen && (
        <div
          className="fixed inset-0 z-30 bg-black/60 md:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      <div
        className={cn(
          'fixed inset-y-0 left-0 z-40 transition-transform duration-200 ease-in-out',
          'md:static md:z-auto md:translate-x-0',
          sidebarOpen ? 'translate-x-0' : '-translate-x-full',
        )}
      >
        <BotSidebar onClose={() => setSidebarOpen(false)} />
      </div>

      <main className="flex-1 overflow-hidden flex flex-col min-w-0">
        <div className="shrink-0 flex items-center gap-2 px-3 h-9 border-b border-border md:hidden">
          <button
            onClick={() => setSidebarOpen(true)}
            className="p-1 rounded hover:bg-muted transition-colors"
          >
            <Menu className="w-4 h-4" />
          </button>
          {selected && (
            <span className="text-xs font-medium truncate">{selected.username}</span>
          )}
        </div>

        {selected ? (
          <BotDetail bot={selected} />
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center gap-3 text-muted-foreground select-none">
            <Monitor className="w-10 h-10 opacity-20" />
            <p className="text-xs">Select a bot to view details</p>
          </div>
        )}
      </main>
    </div>
  )
}
