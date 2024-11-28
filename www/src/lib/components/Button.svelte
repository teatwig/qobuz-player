<script lang="ts">
	export let onClick = null,
		type: 'submit' | 'reset' | 'button' | undefined = undefined;
</script>

<button class="px-4 py-2 text-4xl leading-none" {type} on:click|stopPropagation={onClick}>
	<slot />
</button>
