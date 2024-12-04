<script lang="ts">
	import { page } from '$app/stores';
	import ListTracks from '$lib/components/ListTracks.svelte';
	import Spinner from '$lib/components/Spinner.svelte';
	import { controls } from '$lib/store';
	import { onMount } from 'svelte';
	import { writable, type Writable } from 'svelte/store';
	import { Icon, Star, Play } from 'svelte-hero-icons';

	let id: Writable<number> = writable();

	onMount(() => {
		$id = Number($page.url.searchParams.get('id'));

		if ($id) {
			$controls?.isFavoritePlaylist($id).then((result) => (isFavorite = result));
		}
	});

	function toggle_favorite() {
		if ($id) {
			$controls
				?.set_favorite($id.toString(), !isFavorite, 'playlist')
				.then((response) => (isFavorite = response));
		}
	}

	const playlist = $derived($controls?.playlist($id));
	let isFavorite = $state(false);
</script>

{#await playlist}
	<div class="flex w-full justify-center p-4">
		<Spinner />
	</div>
{:then playlist}
	{#if playlist}
		<div class="flex h-full flex-col items-center justify-center landscape:flex-row">
			{#if playlist.coverArt}
				<div class="landscape::max-w-[50%] flex justify-center p-8 portrait:max-h-[50%]">
					<div class="aspect-square max-h-full overflow-clip rounded-lg shadow-lg">
						<img src={playlist.coverArt} alt={playlist.title} class="object-contain" />
					</div>
				</div>
			{/if}

			<div class="flex h-full w-full flex-col items-center gap-4 overflow-auto">
				<div class="flex w-full flex-col items-center gap-2 text-center">
					<span class="text-lg">{playlist.title}</span>
				</div>

				<div class="flex gap-4">
					<button
						class="flex items-center gap-2 rounded bg-blue-500 px-4 py-2"
						onclick={() => $controls.playPlaylist(playlist.id)}
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
					<ListTracks showTrackNumber={true} tracks={playlist?.tracks ?? []} />
				</div>
			</div>
		</div>
	{/if}
{/await}
