<script lang="ts">
  import type { PageData } from "./$types";
  import dayjs from "$lib/dayjs";
  import { listTracks } from "$lib/api";
  import { fade } from "svelte/transition";

  const numberFormat = new Intl.NumberFormat();

  export let data: PageData;
  $: tracksData = data.tracksData;

  let isLoading: boolean = false;

  const loadMore = async () => {
    isLoading = true;
    try {
      // TODO handle invalidate
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

<div class="overflow-x-auto my-4">
  <table class="table table-sm table-responsive table-fixed">
    <thead>
      <tr>
        <th class="w-48">First Played</th>
        <th>Artist</th>
        <th>Title</th>
        <th class="w-16 sm:text-right">Play <br class="hidden sm:block" />Count</th>
        <th class="w-32 sm:text-right">Last Played</th>
      </tr>
    </thead>
    <tbody>
      {#each tracksData.tracks as track}
        <tr transition:fade>
          <td class="max-sm:font-bold"
            >{dayjs(track.created_at).format("MMM DD, YYYY [at] HH:mm")}</td
          >
          <td>{track.artist}</td>
          <td>
            <a class="link" href={`/station/${data.station.id}/track/${track.id}`}>{track.title}</a>
          </td>
          <td class="sm:text-right">
            {numberFormat.format(track.play_count)} <span class="inline sm:hidden">plays</span>
          </td>
          <td class="sm:text-right">{dayjs(track.updated_at).fromNow()}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<div class="mb-8 h-8 text-center">
  {#if isLoading}
    <span class="loading loading-spinner loading-sm"></span>
  {:else if tracksData.nextToken}
    <button class="btn btn-sm" on:click|preventDefault={loadMore}>Load More</button>
  {:else}
    <div class="badge badge-neutral badge-xs"></div>
  {/if}
</div>
