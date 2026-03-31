import { useState, useEffect, useCallback, useRef, memo } from "react";
import { Search, ChevronLeft, ChevronRight, Loader2 } from "lucide-react";
import { api, type ItemsPage, type ItemRecord } from "@/lib/api";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { TextureImage } from "@/components/texture-image";

export function ItemsPage() {
  const [q, setQ] = useState("");
  const [page, setPage] = useState(1);
  const [data, setData] = useState<ItemsPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [selectedItem, setSelectedItem] = useState<ItemRecord | null>(null);
  const [showDialog, setShowDialog] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const fetchItems = useCallback(async (p: number, query: string) => {
    setLoading(true);
    try {
      const result = await api.getItems(p, query);
      setData(result);
    } catch {
    } finally {
      setLoading(false);
    }
  }, []);

  const qRef = useRef(q);
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    const isQueryChange = qRef.current !== q;
    qRef.current = q;
    if (isQueryChange) {
      setPage(1);
      debounceRef.current = setTimeout(() => fetchItems(1, q), 300);
      return () => {
        if (debounceRef.current) clearTimeout(debounceRef.current);
      };
    } else {
      fetchItems(page, q);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [q, page]);

  const totalPages = data ? Math.ceil(data.total / data.page_size) : 0;

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
        {loading && (
          <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />
        )}
        {data && !loading && (
          <span className="text-xs text-muted-foreground shrink-0">
            {data.total.toLocaleString()} items
          </span>
        )}
      </div>

      <ScrollArea className="flex-1 min-h-0">
        <div 
          className="p-6 grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 gap-4"
          style={{ 
            contentVisibility: 'auto',
            contain: 'layout style paint'
          }}
        >
          {data?.items.map((item) => (
            <ItemCard 
              key={item.id} 
              item={item}
              onClick={() => {
                setSelectedItem(item);
                setShowDialog(true);
              }}
            />
          ))}
          {!loading && data?.items.length === 0 && (
            <div className="col-span-full py-12 text-center text-muted-foreground">
              No items found.
            </div>
          )}
        </div>
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
          {data ? `Page ${page} / ${totalPages || 1}` : "—"}
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

      <Dialog open={showDialog} onOpenChange={setShowDialog}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-3">
              <div className="w-16 h-16 bg-muted rounded overflow-hidden flex-shrink-0">
                {selectedItem && (
                  <TextureImage item={selectedItem} />
                )}
              </div>
              <div>
                <div className="text-xs text-muted-foreground font-mono">
                  #{selectedItem?.id}
                </div>
                <div className="text-lg font-semibold">
                  {selectedItem?.name}
                </div>
              </div>
            </DialogTitle>
          </DialogHeader>
          
          {selectedItem && (
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="space-y-3">
                <div>
                  <span className="text-muted-foreground">ID:</span>
                  <span className="ml-2 font-mono">{selectedItem.id}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Name:</span>
                  <span className="ml-2">{selectedItem.name}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Rarity:</span>
                  <span className="ml-2">{selectedItem.rarity}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Max Item:</span>
                  <span className="ml-2">{selectedItem.max_item}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Render Type:</span>
                  <span className="ml-2 font-mono">{selectedItem.render_type}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Action Type:</span>
                  <span className="ml-2 font-mono">{selectedItem.action_type}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Material:</span>
                  <span className="ml-2 font-mono">{selectedItem.material}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Collision Type:</span>
                  <span className="ml-2 font-mono">{selectedItem.collision_type}</span>
                </div>
              </div>

              <div className="space-y-3">
                <div>
                  <span className="text-muted-foreground">Texture File:</span>
                  <span className="ml-2 text-xs font-mono truncate block">{selectedItem.texture_file_name}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Texture Coords:</span>
                  <span className="ml-2 font-mono">({selectedItem.texture_x}, {selectedItem.texture_y})</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Grow Time:</span>
                  <span className="ml-2">{selectedItem.grow_time ? `${selectedItem.grow_time}s` : '—'}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Drop Chance:</span>
                  <span className="ml-2">{selectedItem.drop_chance}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Block Health:</span>
                  <span className="ml-2">{selectedItem.block_health}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Visual Effect:</span>
                  <span className="ml-2 font-mono">{selectedItem.visual_effect}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Clothing Type:</span>
                  <span className="ml-2 font-mono">{selectedItem.clothing_type}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Flags:</span>
                  <span className="ml-2 font-mono">0x{selectedItem.flags.toString(16).toUpperCase()}</span>
                </div>
              </div>

              {selectedItem.description && (
                <div className="col-span-2">
                  <span className="text-muted-foreground">Description:</span>
                  <p className="mt-1 p-2 bg-muted rounded text-sm">{selectedItem.description}</p>
                </div>
              )}

              {selectedItem.pet_name && (
                <div className="col-span-2 p-2 bg-muted rounded">
                  <span className="text-muted-foreground text-xs">Pet Info:</span>
                  <div className="space-y-1 text-xs">
                    <div>Name: {selectedItem.pet_name}</div>
                    <div>Prefix: {selectedItem.pet_prefix}</div>
                    <div>Suffix: {selectedItem.pet_suffix}</div>
                    <div>Ability: {selectedItem.pet_ability}</div>
                  </div>
                </div>
              )}

              {selectedItem.extra_options && (
                <div className="col-span-2">
                  <span className="text-muted-foreground text-xs">Extra Options:</span>
                  <div className="mt-1 p-2 bg-muted rounded text-xs font-mono truncate">
                    {selectedItem.extra_options}
                  </div>
                </div>
              )}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}

const ItemCard = memo(function ItemCard({ 
  item, 
  onClick 
}: { 
  item: ItemRecord; 
  onClick: () => void;
}) {
  const cardRef = useRef<HTMLDivElement>(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setIsVisible(true);
            observer.disconnect();
          }
        });
      },
      { rootMargin: '200px' }
    );

    if (cardRef.current) {
      observer.observe(cardRef.current);
    }

    return () => observer.disconnect();
  }, []);

  return (
    <div
      ref={cardRef}
      className="p-3 border rounded bg-card hover:shadow-md transition-shadow cursor-pointer"
      style={{ 
        willChange: 'transform',
        contain: 'layout style paint'
      }}
      onClick={onClick}
    >
      <div className="flex items-center justify-center w-full aspect-square bg-muted rounded overflow-hidden mb-3">
        {isVisible ? (
          <TextureImage item={item} />
        ) : (
          <div 
            className="w-full h-full flex items-center justify-center text-muted-foreground text-xs"
            style={{ minHeight: '128px' }}
          >
            ...
          </div>
        )}
      </div>

      <div className="space-y-1">
        <div className="flex items-center gap-2">
          <span className="font-mono text-xs text-muted-foreground">
            #{item.id}
          </span>
        </div>
        <p className="text-sm font-medium truncate" title={item.name}>
          {item.name}
        </p>
        <div className="flex flex-col gap-0.5 text-xs text-muted-foreground">
          <span>Rarity: {item.rarity}</span>
          <span>Max: {item.max_item}</span>
          {item.grow_time && <span>Grow: {item.grow_time}s</span>}
        </div>
      </div>
    </div>
  );
});
