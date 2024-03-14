<script lang="ts">
  export let data: any;
</script>

<h1>Welcome to radiojournal</h1>

<h2>Stations</h2>
<ul>
  {#each data.stations as station}
    <li>
      <a href={`/station/${station.id}`}>{station.name}</a>
      ({station.play_count} plays, {station.track_count} songs)
    </li>
  {/each}
</ul>
