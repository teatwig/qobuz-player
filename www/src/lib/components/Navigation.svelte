<script>
	import { Icon, QueueList, PlayCircle, MagnifyingGlass, Star } from 'svelte-hero-icons';

	export let setPage, activePage;
</script>

<nav class="w-full bg-black *:h-8 flex justify-between py-4 px-8 *:transition-colors">
	<button
		class={$activePage === 'playing' ? 'text-blue-500' : 'text-gray-500'}
		on:click={setPage('playing')}
	>
		<Icon src={PlayCircle} solid />
	</button>
	<button
		class={$activePage == 'queue' ? 'text-blue-500' : 'text-gray-500'}
		on:click={setPage('queue')}
	>
		<Icon src={QueueList} solid />
	</button>
	<button
		class={$activePage == 'favorites' ? 'text-blue-500' : 'text-gray-500'}
		on:click={setPage('favorites')}
	>
		<Icon src={Star} solid />
	</button>
	<button
		class={$activePage == 'search' ? 'text-blue-500' : 'text-gray-500'}
		on:click={setPage('search')}
	>
		<Icon src={MagnifyingGlass} solid />
	</button>
</nav>
