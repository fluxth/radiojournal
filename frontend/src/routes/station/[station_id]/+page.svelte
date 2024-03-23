<script lang="ts">
  import { invalidate } from "$app/navigation";

  export let data: any;

  const loadMore = async (nextToken: string) => {
    const res = await fetch(
      `https://nna6sr3v62fsk5oauz37tnzuvm0lvtfl.lambda-url.ap-southeast-1.on.aws/v1/station/${data.station.id}/plays?next_token=${nextToken}`,
    );

    const { plays, next_token: newNextToken } = await res.json();
    data.content = {
      plays: [...data.content.plays, ...plays],
      nextToken: newNextToken,
    };
  };

  const refresh = async () => {
    await invalidate(
      `https://nna6sr3v62fsk5oauz37tnzuvm0lvtfl.lambda-url.ap-southeast-1.on.aws/v1/station/${data.station.id}/plays`,
    );
  };
</script>

<svelte:head>
  <title>{data.station.name} - radiojournal</title>
</svelte:head>

<div class="px-2 py-6 flex flex-wrap gap-4">
  <h2 class="font-bold text-2xl truncate">{data.station.name}</h2>
  <button class="btn btn-sm" on:click|preventDefault={refresh}>Refresh</button>
</div>

<div class="text-sm breadcrumbs px-4 bg-base-200 rounded-md">
  <ul>
    <li><a href="/">Stations</a></li>
    <li>{data.station.name}</li>
  </ul>
</div>

<div class="overflow-x-auto my-4">
  <table class="table table-sm">
    <thead>
      <tr>
        <th>Timestamp</th>
        <th>Artist</th>
        <th>Title</th>
      </tr>
    </thead>
    <tbody>
      {#each data.content.plays as play}
        <tr class={play.track.is_song ? "" : "italic text-neutral-300 dark:text-neutral-600"}>
          <td>{new Date(play.played_at).toLocaleString()}</td>
          <td>{play.track.artist}</td>
          <td>{play.track.title}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<div class="mb-6 flex justify-center">
  <button
    class="btn btn-sm btn-primary"
    disabled={!data.content.nextToken}
    on:click={() => loadMore(data.content.nextToken)}
  >
    Load More
  </button>
</div>
