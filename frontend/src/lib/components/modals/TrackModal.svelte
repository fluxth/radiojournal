<script lang="ts">
  import { type TrackMinimal, getTrack } from "$lib/api";
  import dayjs from "$lib/dayjs";

  let dialog: HTMLDialogElement;

  const numberFormat = new Intl.NumberFormat();

  export let stationId: string;
  export let trackMinimal: TrackMinimal | null;
  $: trackPromise = trackMinimal ? getTrack({ stationId, trackId: trackMinimal.id }) : null;

  export const show = () => {
    dialog.showModal();
  };
</script>

<dialog class="modal" bind:this={dialog}>
  <div class="modal-box">
    <form method="dialog">
      <button class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2">✕</button>
    </form>
    <h3 class="font-bold text-lg -mt-2 mb-4">Track Details</h3>

    {#if trackMinimal && trackPromise}
      <div class="overflow-x-auto">
        <table class="table">
          <tbody>
            <tr>
              <td class="font-bold">Track ID</td>
              <td>{trackMinimal.id}</td>
            </tr>
            <tr>
              <td class="font-bold">Artist</td>
              <td>{trackMinimal.artist}</td>
            </tr>
            <tr>
              <td class="font-bold">Title</td>
              <td>{trackMinimal.title}</td>
            </tr>
            <tr>
              <td class="font-bold">Type</td>
              <td>{trackMinimal.is_song ? "Music" : "Other"}</td>
            </tr>
            <tr>
              <td class="font-bold">Created</td>
              <td>
                {#await trackPromise}
                  <div class="skeleton h-3 w-full"></div>
                {:then track}
                  <div class="tooltip tooltip-top" data-tip={track.created_at}>
                    <button class="underline decoration-dashed cursor-help">
                      {dayjs(track.created_at).fromNow()}
                    </button>
                  </div>
                {/await}
              </td>
            </tr>
            <tr>
              <td class="font-bold">Last Played</td>
              <td>
                {#await trackPromise}
                  <div class="skeleton h-3 w-full"></div>
                {:then track}
                  <div class="tooltip tooltip-top" data-tip={track.updated_at}>
                    <button class="underline decoration-dashed cursor-help">
                      {dayjs(track.updated_at).fromNow()}
                    </button>
                  </div>
                {/await}
              </td>
            </tr>
            <tr>
              <td class="font-bold">Play Count</td>
              <td>
                {#await trackPromise}
                  <div class="skeleton h-3 w-full"></div>
                {:then track}
                  {numberFormat.format(track.play_count)}
                {/await}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    {:else}
      <div role="alert" class="alert alert-error">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="stroke-current shrink-0 h-6 w-6"
          fill="none"
          viewBox="0 0 24 24"
          ><path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
          /></svg
        >
        <span>No track selected</span>
      </div>
    {/if}
  </div>

  <form method="dialog" class="modal-backdrop">
    <button>close</button>
  </form>
</dialog>
