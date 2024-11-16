<script lang="ts">
  import type { PageData } from "./$types";

  import TracksTable from "$lib/components/TracksTable.svelte";
  import { listTracks } from "$lib/api";

  const numberFormat = new Intl.NumberFormat();

  type Props = {
    data: PageData;
  };

  let { data }: Props = $props();
  let tracksData = $state(data.tracksData);
  let isLoading = $state(false);

  const loadMore = async (event: MouseEvent) => {
    event.preventDefault();

    isLoading = true;
    try {
      // TODO handle invalidate
      const { tracks, ...rest } = await listTracks({
        stationId: data.station.id,
        nextToken: tracksData.nextToken,
        artist: data.artist.name,
      });

      tracksData = { tracks: [...tracksData.tracks, ...tracks], ...rest };
    } finally {
      isLoading = false;
    }
  };
</script>

<svelte:head>
  <title>{data.artist.name} - {data.station.name} - radiojournal</title>
</svelte:head>

<div class="px-2 py-6 flex flex-wrap gap-4">
  <h2 class="font-bold text-2xl truncate">{data.station.name}</h2>
</div>

<div class="text-sm breadcrumbs px-4 bg-base-200 rounded-md">
  <ul>
    <li><a href="/">Stations</a></li>
    <li><a href={`/station/${data.station.id}/plays`}>{data.station.name}</a></li>
    <li><a href={`/station/${data.station.id}/tracks`}>Tracks</a></li>
    <li>{data.artist.name}</li>
  </ul>
</div>

<h2 class="text-2xl truncate mx-2 my-4">
  <span class="font-bold">Tracks: {data.artist.name}</span>
  ({numberFormat.format(data.tracksData.tracks.length)}{data.tracksData.nextToken ? "+" : ""})
</h2>

<TracksTable station={data.station} tracks={tracksData.tracks} artistLink={false} />

<div class="mb-8 h-8 text-center">
  {#if isLoading}
    <span class="loading loading-spinner loading-sm"></span>
  {:else if tracksData.nextToken}
    <button class="btn btn-sm" onclick={loadMore}>Load More</button>
  {:else}
    <div class="badge badge-neutral badge-xs"></div>
  {/if}
</div>
