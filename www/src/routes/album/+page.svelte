<script lang="ts">
	import { page } from '$app/stores';
	import ListTracks from '$lib/components/ListTracks.svelte';
	import Spinner from '$lib/components/Spinner.svelte';
	import { controls } from '$lib/store';
	import { onMount } from 'svelte';
	import { writable, type Writable } from 'svelte/store';
	import { Icon, Play, Star } from 'svelte-hero-icons';

	let id: Writable<string | undefined> = writable();

	onMount(() => {
		$id = $page.url.searchParams.get('id') ?? undefined;

		if ($id) {
			$controls?.isFavoriteAlbum($id).then((result) => (isFavorite = result));
		}
	});

	function toggle_favorite() {
		if ($id) {
			$controls
				?.set_favorite($id, !isFavorite, 'album')
				.then((response) => (isFavorite = response));
		}
	}

	const album = $derived($controls?.album($id));

	let isFavorite = $state(false);
</script>

{#await album}
	<div class="flex w-full justify-center p-4">
		<Spinner />
	</div>
{:then album}
	{#if album}
		<div class="flex h-full flex-col items-center justify-center landscape:flex-row">
			{#if album.coverArt}
				<div class="landscape::max-w-[50%] flex justify-center p-8 portrait:max-h-[50%]">
					<div class="aspect-square max-h-full overflow-clip rounded-lg shadow-lg">
						<img src={album.coverArt} alt={album.title} class="object-contain" />
					</div>
				</div>
			{/if}

			<div class="flex w-full flex-col items-center gap-4 overflow-auto">
				<div class="flex w-full flex-col items-center gap-2 text-center">
					<span class="w-full truncate text-lg">{album.title}</span>
					<span class="text-gray-400">{album.releaseYear}</span>
				</div>

				<div class="flex gap-4">
					<button
						class="flex items-center gap-2 rounded bg-blue-500 px-4 py-2"
						onclick={() => $controls.playAlbum(album.id)}
					>
						<Icon src={Play} class="size-6" solid />
						<span>Play</span>
					</button>

					<button
						onclick={() => toggle_favorite()}
						class="flex items-center gap-2 rounded bg-blue-500 px-4 py-2"
					>
						<Icon src={Star} class="size-6" solid={isFavorite} />
						<span>Favorite</span>
					</button>
				</div>

				<div class="w-full max-w-screen-sm">
					<ListTracks showTrackNumber={true} tracks={album?.tracks ?? []} />
				</div>
			</div>
		</div>
	{/if}
{/await}
