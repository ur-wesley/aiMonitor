import { type JSX, Show, splitProps } from "solid-js";

import { getCurrentWindow } from "@tauri-apps/api/window";



import { cn } from "~/lib/utils";



type WindowFrameProps = {

  readonly title: string;

  readonly class?: string;

  readonly footer?: JSX.Element;

  readonly children?: JSX.Element;

};



export function WindowFrame(props: WindowFrameProps) {

  const [local, rest] = splitProps(props, ["title", "class", "footer"]);



  const hideWindow = () => {

    void getCurrentWindow().hide();

  };



  return (

    <div class={cn("flex h-screen flex-col overflow-hidden p-2", local.class)}>

      <div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[10px] border border-border bg-bg/70 shadow-xl backdrop-blur-xl">

        <header class="window-titlebar flex h-10 shrink-0 items-stretch rounded-t-[10px] border-b border-border bg-surface">

          <div

            class="flex min-w-0 flex-1 items-center gap-2 px-3"

            data-tauri-drag-region

          >

            <span class="i-[mdi--cog] size-4 shrink-0 text-accent" aria-hidden="true" />

            <h1 class="truncate text-sm font-semibold">{local.title}</h1>

          </div>

          <div class="window-controls flex h-full shrink-0 items-stretch border-l border-border">

            <button

              type="button"

              class="inline-flex w-12 items-center justify-center border-l border-border text-lg leading-none text-muted transition-colors hover:bg-critical hover:text-white"

              onClick={hideWindow}

              aria-label="Close"

            >

              <span aria-hidden="true">×</span>

            </button>

          </div>

        </header>

        <div class="app-scroll app-scroll-y min-h-0 flex-1 p-3">{rest.children}</div>

        <Show when={local.footer}>

          <footer class="shrink-0 border-t border-border px-3 py-2">{local.footer}</footer>

        </Show>

      </div>

    </div>

  );

}


