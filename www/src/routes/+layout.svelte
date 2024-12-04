<script lang="ts">
	import '../app.postcss';

	import { onMount } from 'svelte';
	import { Controls } from '$lib/controls';
	import { isBuffering, isLoading, currentStatus, connected } from '$lib/store';
	import { dev } from '$app/environment';
	import Navigation from '../lib/components/Navigation.svelte';
	import { Icon, LinkSlash } from 'svelte-hero-icons';
	import Spinner from '../lib/components/Spinner.svelte';
	import { controls } from '$lib/store';

	let isInitialized = true;

	onMount(() => {
		$controls = new Controls(dev);
		isInitialized = false;

		const onFocus = () => {
			if (!$connected) {
				$controls?.connect();
			}
		};

		window.addEventListener('focus', onFocus);
	});
</script>

<svelte:head>
	<title>hifi.rs: {$currentStatus}</title>
</svelte:head>

<div class="flex h-full flex-col justify-between px-safe pt-safe">
	<div class="flex h-full flex-col justify-between overflow-hidden">
		{#if isInitialized}
			<div class="flex w-full justify-center p-4">
				<Spinner />
			</div>
		{:else}
			<slot />
		{/if}
	</div>

	<Navigation />
</div>

{#if $isBuffering || !$connected || $isLoading}
	<div class="fixed right-8 top-8 z-10 size-12 rounded bg-black bg-opacity-20 p-2 backdrop-blur">
		{#if !$connected}
			<Icon solid src={LinkSlash} />
		{:else if $isLoading || $isBuffering}
			<Spinner />
		{/if}
	</div>
{/if}
