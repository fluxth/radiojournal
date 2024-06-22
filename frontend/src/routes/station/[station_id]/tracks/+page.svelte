<script lang="ts">
  import type { PageData } from "./$types";
  import { listTracks } from "$lib/api";
  import TracksTable from "$lib/components/TracksTable.svelte";

  const numberFormat = new Intl.NumberFormat();

  export let data: PageData;
  $: tracksData = data.tracksData;

  let isLoading: boolean = false;

  const loadMore = async () => {
    isLoading = true;
    try {
      // TODO: handle invalidate
      const { tracks, ...rest } = await listTracks({
        stationId: data.station.id,
        nextToken: tracksData.nextToken,
      });

      tracksData = { tracks: [...tracksData.tracks, ...tracks], ...rest };
    } finally {
      isLoading = false;
    }
  };
</script>

<svelte:head>
  <title>Tracks - {data.station.name} - radiojournal</title>
</svelte:head>

<div class="px-2 py-6 flex flex-wrap gap-4">
  <h2 class="font-bold text-2xl truncate">{data.station.name}</h2>
</div>

<div class="text-sm breadcrumbs px-4 bg-base-200 rounded-md">
  <ul>
    <li><a href="/">Stations</a></li>
    <li><a href={`/station/${data.station.id}/plays`}>{data.station.name}</a></li>
    <li>Tracks</li>
  </ul>
</div>

<h2 class="text-2xl truncate mx-2 my-4">
  <span class="font-bold">Tracks</span> ({numberFormat.format(data.station.track_count)})
</h2>

<TracksTable station={data.station} tracks={tracksData.tracks} />

<div class="mb-8 h-8 text-center">
  {#if isLoading}
    <span class="loading loading-spinner loading-sm"></span>
  {:else if tracksData.nextToken}
    <button class="btn btn-sm" on:click|preventDefault={loadMore}>Load More</button>
  {:else}
    <div class="badge badge-neutral badge-xs"></div>
  {/if}
</div>
