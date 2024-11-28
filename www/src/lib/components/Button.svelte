<script>
	export let onClick = null,
		type = null;
</script>

<button {type} class="px-4 py-2 text-4xl leading-none" on:click|stopPropagation={onClick}>
	<slot />
</button>
