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
    <span class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-slate-400 dark:text-slate-500">
      <span class="fa-solid fa-magnifying-glass"></span>
    </span>
    <input
      class="block w-full rounded border-2 border-slate-300 bg-white px-3 py-2 pl-10 pr-20 text-base text-slate-900 placeholder:text-slate-500 transition focus:border-orange-500 focus:outline-none focus:ring-2 focus:ring-orange-300 disabled:cursor-not-allowed disabled:opacity-60 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100 dark:placeholder-slate-400"
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
        class="absolute inset-y-0 right-3 flex items-center gap-1 rounded bg-slate-100 px-3 text-xs font-semibold uppercase tracking-wide text-slate-600 transition hover:bg-slate-200 hover:text-slate-700 dark:bg-slate-700 dark:text-slate-300 dark:hover:bg-slate-600 dark:hover:text-slate-100"
        on:click={handleClear}
        aria-label="Clear search"
      >
        <span class="fa-solid fa-xmark text-slate-400 dark:text-slate-500"></span>
        Clear
      </button>
    {/if}
  </div>
  <button
    type="button"
    class="flex items-center gap-2 rounded bg-orange-500 px-4 py-2 text-sm font-semibold uppercase tracking-wide text-white transition hover:bg-orange-600 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-orange-500 dark:hover:bg-orange-600"
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
