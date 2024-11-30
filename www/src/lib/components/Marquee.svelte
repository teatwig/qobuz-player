<script lang="ts">
	let { input } = $props<{ input: string }>();

	let titleWidth = $state(0);
	let titleWrapperWidth = $state(0);

	const enableMarquee = $derived(titleWidth > titleWrapperWidth);
</script>

<div class="flex overflow-hidden" bind:offsetWidth={titleWrapperWidth}>
	<span class="whitespace-nowrap" class:marquee={enableMarquee} bind:offsetWidth={titleWidth}>
		{input}
	</span>

	{#if enableMarquee}
		<span class="marquee extra-element whitespace-nowrap">
			{input}
		</span>
	{/if}
</div>

<style lang="postcss">
	@media (prefers-reduced-motion) {
		.marquee {
			overflow: hidden;
			text-overflow: ellipsis;
			white-space: nowrap;
		}
		.extra-element {
			display: none;
		}
	}

	@media not (prefers-reduced-motion) {
		.marquee {
			padding-right: 2rem;
			animation-name: marquee;
			animation-duration: 15s;
			animation-iteration-count: infinite;
			animation-timing-function: cubic-bezier(0.3, 0, 0.7, 1);
		}

		@keyframes marquee {
			0% {
				transform: translateX(0%);
			}

			20% {
				transform: translateX(0%);
			}

			100% {
				transform: translateX(-100%);
			}
		}
	}
</style>
