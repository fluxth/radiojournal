<script lang="ts">
  import type { Station, Track } from "$lib/api";

  import { fade } from "svelte/transition";
  import dayjs from "$lib/dayjs";

  const numberFormat = new Intl.NumberFormat();

  type Props = {
    tracks: Track[];
    station: Station;
    artistLink?: boolean;
  };

  let { tracks, station, artistLink = true }: Props = $props();
</script>

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
      {#each tracks as track (track.id)}
        <tr transition:fade>
          <td class="max-sm:font-bold">
            {dayjs(track.created_at).format("MMM DD, YYYY [at] HH:mm")}
          </td>
          <td>
            {#if artistLink}
              <a
                class="link"
                href={`/station/${station.id}/artist/${encodeURIComponent(track.artist)}`}
              >
                {track.artist}
              </a>
            {:else}
              {track.artist}
            {/if}
          </td>
          <td>
            <a class="link" href={`/station/${station.id}/track/${track.id}`}>{track.title}</a>
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
