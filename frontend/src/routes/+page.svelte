<script lang="ts">
  import type { PageData } from "./$types";

  import { invalidateAll } from "$app/navigation";

  export let data: PageData;

  const numberFormat = new Intl.NumberFormat();

  const refresh = async () => {
    await invalidateAll();
  };
</script>

<svelte:head>
  <title>Stations - radiojournal</title>
</svelte:head>

<div class="px-2 py-6 flex flex-wrap gap-4">
  <h2 class="font-bold text-2xl truncate">Stations</h2>
  <button class="btn btn-sm" on:click|preventDefault={refresh}>Refresh</button>
</div>

<div class="grid md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 mb-6">
  {#each data.stations as station}
    <div class="card bg-base-200 shadow-lg">
      <div class="card-body">
        <div>
          <h2 class="card-title">{station.name}</h2>
          {#if station.location}
            <p class="text-xs italic">{station.location}</p>
          {/if}
        </div>
        <p>
          {numberFormat.format(station.play_count)} plays &middot;
          {numberFormat.format(station.track_count)} tracks
        </p>
        <div class="card-actions justify-end mt-2">
          <a class="btn btn-neutral" href={`/station/${station.id}/tracks`}>Track List</a>
          <a class="btn btn-primary" href={`/station/${station.id}/plays`}>Play History</a>
        </div>
      </div>
    </div>
  {/each}
</div>
