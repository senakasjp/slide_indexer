<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let id: string;
  export let value: string;
  export let placeholder: string;
  export let isSearching = false;

  const dispatch = createEventDispatcher<{ change: string; clear: void; submit: void }>();

  $: trimmedValue = value.trim();

  const handleInput = (event: Event) => {
    const target = event.target as HTMLInputElement | null;
    dispatch('change', target?.value ?? '');
  };

  const handleClear = () => {
    dispatch('clear');
  };

  const handleSubmit = () => {
    dispatch('submit');
  };
</script>

<div class="flex w-full items-center gap-3">
  <div class="relative flex-1">
    <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4">
      <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-orange-100 to-rose-100 dark:from-orange-900/30 dark:to-rose-900/30">
        <span class="fa-solid fa-magnifying-glass text-sm text-orange-600 dark:text-orange-400"></span>
      </div>
    </div>
    <input
      class="block w-full rounded-xl border-2 border-slate-200 bg-white/80 backdrop-blur-sm px-4 py-3 pl-14 pr-24 text-base text-slate-900 placeholder:text-slate-500 shadow-sm transition-all duration-200 focus:border-orange-400 focus:outline-none focus:ring-4 focus:ring-orange-200/50 focus:shadow-lg disabled:cursor-not-allowed disabled:opacity-60 dark:border-slate-600 dark:bg-slate-800/80 dark:text-slate-100 dark:placeholder-slate-400 dark:focus:border-orange-500 dark:focus:ring-orange-500/30"
      type="search"
      id={id}
      {placeholder}
      {value}
      on:input={handleInput}
      on:keydown={(event) => {
        if (event.key === 'Enter') {
          event.preventDefault();
          handleSubmit();
        }
      }}
      autocomplete="off"
      autocapitalize="none"
      spellcheck="false"
    />
    {#if value}
      <button
        type="button"
        class="absolute inset-y-0 right-3 my-auto flex h-8 items-center gap-1.5 rounded-lg bg-slate-100 px-3 text-xs font-semibold uppercase tracking-wide text-slate-600 shadow-sm transition-all hover:bg-slate-200 hover:text-slate-700 hover:shadow dark:bg-slate-700 dark:text-slate-300 dark:hover:bg-slate-600 dark:hover:text-slate-100"
        on:click={handleClear}
        aria-label="Clear search"
      >
        <span class="fa-solid fa-xmark"></span>
        Clear
      </button>
    {/if}
  </div>
  <button
    type="button"
    class="flex items-center gap-2 rounded-xl bg-gradient-to-r from-orange-500 to-rose-500 px-6 py-3 text-sm font-bold uppercase tracking-wide text-white shadow-lg shadow-orange-500/30 transition-all duration-300 hover:from-orange-600 hover:to-rose-600 hover:shadow-xl hover:shadow-orange-500/40 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:shadow-lg dark:from-orange-500 dark:to-rose-500 dark:hover:from-orange-600 dark:hover:to-rose-600"
    on:click={handleSubmit}
    disabled={!trimmedValue || isSearching}
  >
    {#if isSearching}
      <span class="fa-solid fa-circle-notch fa-spin"></span>
    {:else}
      <span class="fa-solid fa-magnifying-glass"></span>
    {/if}
    Search
  </button>
</div>
