<script lang="ts">
	import { writable } from 'svelte/store';
	import { controls } from '$lib/store';
	import ListAlbums from '$lib/components/ListAlbums.svelte';
	import Spinner from '$lib/components/Spinner.svelte';
	import ListArtists from '$lib/components/ListArtists.svelte';
	import ListPlaylists from '$lib/components/ListPlaylists.svelte';

	const favorites = $derived($controls?.favorites());
	const favoritePlaylists = $derived($controls?.favoritePlaylists());

	const tab = writable<'albums' | 'artists' | 'playlists'>('albums');
</script>

<div class="flex max-h-full flex-grow flex-col">
	<div class="flex flex-col gap-4 p-4">
		<h1 class="text-2xl">Favorites</h1>

		<div class="flex justify-between *:rounded-full *:px-2 *:py-1 *:transition-colors">
			<button class:bg-blue-800={$tab === 'albums'} onclick={() => tab.set('albums')}>
				Albums
			</button>
			<button class:bg-blue-800={$tab === 'artists'} onclick={() => tab.set('artists')}>
				Artists
			</button>
			<button class:bg-blue-800={$tab === 'playlists'} onclick={() => tab.set('playlists')}>
				Playlists
			</button>
		</div>
	</div>

	{#await favorites}
		<div class="flex w-full justify-center p-4">
			<Spinner />
		</div>
	{:then data}
		{#if $tab === 'albums'}
			<ListAlbums sortBy="artist" albums={data?.albums ?? []} />
		{:else if $tab === 'artists'}
			<ListArtists sortBy="name" artists={data?.artists ?? []} />
		{:else if $tab === 'playlists'}
			{#await favoritePlaylists}
				<div class="flex w-full justify-center p-4">
					<Spinner />
				</div>
			{:then playlists}
				<ListPlaylists sortBy="title" playlists={playlists ?? []} />
			{/await}
		{/if}
	{/await}
</div>
