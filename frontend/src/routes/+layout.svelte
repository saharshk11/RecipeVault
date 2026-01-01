<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
	<link
		rel="stylesheet"
		href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,600;700&family=Space+Grotesk:wght@400;500;600&display=swap"
	/>
</svelte:head>
{@render children()}
