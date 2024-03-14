<script lang="ts">
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
</script>

<a href="/">Back to home</a>

<h1>Station: {data.station.id}</h1>

<table>
  <thead>
    <tr>
      <th>Timestamp</th>
      <th>Artist</th>
      <th>Title</th>
    </tr>
  </thead>
  <tbody>
    {#each data.content.plays as play}
      <tr style={play.track.is_song ? "" : "color: #ccc"}>
        <td>{new Date(play.played_at)}</td>
        <td>{play.track.artist}</td>
        <td>{play.track.title}</td>
      </tr>
    {/each}
  </tbody>
</table>

<button disabled={!data.content.nextToken} on:click={() => loadMore(data.content.nextToken)}
  >Load More</button
>
