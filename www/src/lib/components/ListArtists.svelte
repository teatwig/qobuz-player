<script lang="ts">
	import type { Artist } from '$lib/bindings/Artist';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';

	let { artists, sortBy }: { artists: Artist[]; sortBy: 'default' | 'name' } = $props();

	let sorted = (function () {
		switch (sortBy) {
			case 'default':
				return artists;
			case 'name':
				return artists.sort((a, b) => (a.name < b.name ? -1 : 1));
		}
	})();
</script>

<List>
	{#each sorted as artist}
		<ListItem>
			<a href="/artist?id={artist.id}" class="block truncate text-lg">
				{artist.name}
			</a>
		</ListItem>
	{/each}
</List>
