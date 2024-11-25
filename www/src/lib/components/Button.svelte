<script>
	export let onClick = null,
		type = null;
</script>

<button
	{type}
	class="py-2 px-4 text-4xl leading-none uppercase lg:py-4 lg:px-6 xl:text-3xl"
	on:click|stopPropagation={onClick}
>
	<slot />
</button>
