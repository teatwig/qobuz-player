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

	onMount(() => {
		$controls = new Controls(dev);

		const onFocus = () => {
			if (!$connected) {
				$controls?.connect();
			}
		};

		window.addEventListener('focus', onFocus);

		return () => {
			$controls?.close();
			window.removeEventListener('focus', onFocus);
		};
	});
</script>

<svelte:head>
	<title>hifi.rs: {$currentStatus}</title>
</svelte:head>

<div class="flex h-full flex-col justify-between px-safe pt-safe">
	<div class="flex h-full flex-col justify-between overflow-hidden">
		<slot />
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
