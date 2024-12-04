<script lang="ts">
	import { page } from '$app/stores';
	import { Icon, QueueList, PlayCircle, MagnifyingGlass, Star } from 'svelte-hero-icons';
</script>

<nav
	class="flex w-full justify-evenly gap-2 p-safe *:flex *:h-[3.25rem] *:w-20 *:flex-col *:items-center *:overflow-visible *:text-nowrap *:px-4 *:py-1 *:text-[10px] *:font-medium *:transition-colors"
>
	<a href="/" class={$page.route.id === '/' ? 'text-blue-500' : 'text-gray-500'}>
		<Icon solid src={PlayCircle} />
		Now Playing
	</a>
	<a href="/queue" class={$page.route.id === '/queue' ? 'text-blue-500' : 'text-gray-500'}>
		<Icon solid src={QueueList} />
		Queue
	</a>
	<a href="/favorites" class={$page.route.id === '/favorites' ? 'text-blue-500' : 'text-gray-500'}>
		<Icon solid src={Star} />
		Favorites
	</a>
	<a href="/search" class={$page.route.id === '/search' ? 'text-blue-500' : 'text-gray-500'}>
		<Icon solid src={MagnifyingGlass} />
		Search
	</a>
</nav>
