<script lang="ts">
	import ListAlbums from '$lib/components/ListAlbums.svelte';
	import Spinner from '$lib/components/Spinner.svelte';
	import { controls } from '$lib/store';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { writable, type Writable } from 'svelte/store';
	import { Icon, Star } from 'svelte-hero-icons';

	let id: Writable<number> = writable();

	onMount(() => {
		$id = Number($page.url.searchParams.get('id'));

		if ($id) {
			$controls?.isFavoriteArtist($id).then((result) => (isFavorite = result));
		}
	});

	const artist = $derived($controls.artist($id));
	const albums = $derived($controls.artistAlbumReleases($id));
	let isFavorite = $state(false);

	function toggle_favorite() {
		if ($id) {
			$controls
				?.set_favorite($id.toString(), !isFavorite, 'artist')
				.then((response) => (isFavorite = response));
		}
	}
</script>

{#await artist}
	<div class="flex w-full justify-center p-4">
		<Spinner />
	</div>
{:then artist}
	<div class="flex max-h-full flex-grow flex-col">
		<div class="flex items-center justify-between gap-4 p-4">
			<h1 class="text-2xl">{artist?.name}</h1>

			<button
				onclick={() => toggle_favorite()}
				class="flex items-center gap-2 rounded bg-blue-500 px-4 py-2"
			>
				<Icon src={Star} class="size-6" solid={isFavorite} />
				<span>Favorite</span>
			</button>
		</div>
		{#await albums}
			<div class="flex w-full justify-center p-4">
				<Spinner />
			</div>
		{:then albums}
			<ListAlbums sortBy="releaseYear" albums={albums ?? []} />
		{/await}
	</div>
{/await}
