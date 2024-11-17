<script>
	export let onClick = null,
		type = null;
</script>

<button
	{type}
	class="leading-none px-4 py-2 lg:px-6 lg:py-4 text-4xl xl:text-3xl uppercase"
	on:click|stopPropagation={onClick}
>
	<slot />
</button>
