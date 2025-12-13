<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import { Input } from "$lib/components/ui/input";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { Search, Pin, X, Trash2, Clipboard } from "@lucide/svelte";
  import SunIcon from "@lucide/svelte/icons/sun";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import { toggleMode } from "mode-watcher";
  import { listen } from "@tauri-apps/api/event";
  import { toast } from "svelte-sonner";
  import { fly, scale } from "svelte/transition";
  import hljs from "highlight.js";
  import "highlight.js/styles/atom-one-dark.css";

  let unlisten: (() => void) | null = null;
  let history: ClipboardItem[] = [];
  let searchQuery = "";
  let privateMode = false;

  interface ClipboardItem {
    id: number;
    kind: "text" | "image";
    content: string;
    timestamp: Date;
    pinned?: boolean;
  }

  $: filteredHistory = history
    .filter((item) => {
      if (searchQuery.trim() === "") return true;
      if (item.kind === "text") {
        return item.content.toLowerCase().includes(searchQuery.toLowerCase());
      }
      return false;
    })
    .sort((a, b) => {
      if (a.pinned && !b.pinned) return -1;
      if (!a.pinned && b.pinned) return 1;
      return b.id - a.id;
    });

  onMount(async () => {
    unlisten = await listen<any>("clipboard-changed", (event) => {
      const payload = event.payload;

      if (privateMode) return;
      const existingIndex = history.findIndex(
        (item) => item.content === payload.content
      );
      if (existingIndex !== -1) {
        const isPinned = history[existingIndex].pinned;
        history = history.filter((_, i) => i !== existingIndex);

        const newItem: ClipboardItem = {
          id: Date.now(),
          kind: payload.kind === "image" ? "image" : "text",
          content: payload.content,
          timestamp: new Date(),
          pinned: isPinned,
        };
        history = [newItem, ...history];
        return;
      }

      const newItem: ClipboardItem = {
        id: Date.now(),
        kind: payload.kind === "image" ? "image" : "text",
        content: payload.content,
        timestamp: new Date(),
        pinned: false,
      };

      history = [newItem, ...history];
    });
  });

  onDestroy(() => {
    unlisten?.();
  });

  function clearHistory() {
    history = [];
    toast.success("History cleared");
  }

  function deleteItem(e: Event, id: number) {
    e.stopPropagation();
    history = history.filter((item) => item.id !== id);
  }

  function togglePin(e: Event, id: number) {
    e.stopPropagation();
    history = history.map((item) =>
      item.id === id ? { ...item, pinned: !item.pinned } : item
    );
  }

  async function copyToClipboard(item: ClipboardItem) {
    try {
      if (item.kind === "text") {
        await navigator.clipboard.writeText(item.content);
      } else {
        try {
          const res = await fetch(`data:image/png;base64,${item.content}`);
          const blob = await res.blob();
          await navigator.clipboard.write([
            new ClipboardItem({
              [blob.type]: blob,
            }),
          ]);
        } catch (e) {}
      }
      toast.success("Copied to clipboard");
    } catch (err) {}
  }

  function highlightCode(content: string) {
    if (!content) return "";
    const truncated =
      content.length > 500 ? content.slice(0, 500) + "..." : content;
    return hljs.highlightAuto(truncated).value;
  }
</script>

<div
  class="h-screen w-full flex flex-col bg-background/95 backdrop-blur-sm text-foreground overflow-hidden font-sans"
>
  <div class="p-3 border-b border-border/40 bg-background/50 z-10 space-y-2">
    <div class="relative">
      <Search
        class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground z-10"
      />
      <Input
        type="text"
        bind:value={searchQuery}
        placeholder="Type here to search..."
        class="pl-9 h-9 rounded-full bg-muted/50 border-input shadow-sm focus-visible:ring-1"
      />
    </div>
  </div>

  <ScrollArea class="flex-1 w-full min-h-0 bg-transparent">
    <div class="flex flex-col p-2 gap-1">
      {#each filteredHistory as item (item.id)}
        <div
          role="button"
          tabindex="0"
          onclick={() => copyToClipboard(item)}
          onkeydown={(e) => e.key === "Enter" && copyToClipboard(item)}
          class="group flex items-center gap-3 p-3 rounded-lg
          hover:bg-muted/60 cursor-pointer transition-all border
          hover:border-border/30 relative"
          in:scale={{ duration: 200, start: 0.95 }}
          out:fly={{ x: -20, duration: 200 }}
        >
          <div class="flex-1 min-w-0">
            {#if item.kind === "image"}
              <div
                class="h-16 w-16 rounded overflow-hidden bg-muted border border-border/50 shrink-0"
              >
                <img
                  src={`data:image/png;base64,${item.content}`}
                  alt="Clipboard"
                  class="w-full h-full object-cover"
                />
              </div>
            {:else}
              <div
                class="text-sm text-foreground/90 line-clamp-1
                break-all font-mono leading-relaxed overflow-hidden pointer-events-none"
              >
                {@html highlightCode(item.content)}
              </div>
            {/if}
            <span
              class="text-[10px] text-muted-foreground/50 mt-1 block font-mono"
            >
              {item.timestamp.toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
              })}
            </span>
          </div>

          <div
            class="flex flex-col gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200 shrink-0"
          >
            <Button
              variant="ghost"
              size="icon"
              onclick={(e) => togglePin(e, item.id)}
              class="h-6 w-6 rounded-full hover:bg-background/80 hover:text-primary {item.pinned
                ? 'text-primary opacity-100'
                : 'text-muted-foreground'}"
            >
              <Pin
                class="h-3.5 w-3.5 {item.pinned
                  ? 'fill-current'
                  : ''} {item.pinned ? 'rotate-45' : ''}"
              />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              onclick={(e) => deleteItem(e, item.id)}
              class="h-6 w-6 rounded-full hover:bg-destructive/10 hover:text-destructive text-muted-foreground"
            >
              <X class="h-3.5 w-3.5" />
            </Button>
          </div>

          {#if item.pinned}
            <div
              class="absolute right-2 top-2 opacity-100 group-hover:opacity-0 transition-opacity"
            >
              <Pin class="h-3 w-3 text-primary fill-current rotate-45" />
            </div>
          {/if}
        </div>
      {/each}

      {#if history.length === 0}
        <div
          class="flex flex-col items-center justify-center py-10 text-muted-foreground gap-2 opacity-50"
        >
          <Clipboard class="h-8 w-8 mb-2 opacity-20" />
          <span class="text-xs">Clipboard is empty</span>
        </div>
      {/if}
    </div>
  </ScrollArea>

  <div
    class="p-2 border-t border-border/40 bg-muted/20 backdrop-blur-md mt-auto"
  >
    <div class="flex items-center gap-2">
      <Button
        variant="ghost"
        onclick={clearHistory}
        class="justify-start flex-1 h-8 px-2 text-xs text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
      >
        <Trash2 class="h-3.5 w-3.5 mr-2" />
        Clear history
      </Button>

      <Button
        onclick={toggleMode}
        variant="outline"
        size="icon"
        class="h-8 w-8"
      >
        <SunIcon
          class="transition-all h-[1.2rem] w-[1.2rem] rotate-0 scale-100 dark:-rotate-90 dark:scale-0"
        />
        <MoonIcon
          class="transition-all absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 dark:rotate-0 dark:scale-100"
        />
      </Button>
    </div>
  </div>
</div>
