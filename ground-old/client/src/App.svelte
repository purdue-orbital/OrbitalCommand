<script>
    import Content from './Content.svelte';
    import Modal from 'svelte-simple-modal';
    import {modal} from './stores/ModalStore';
</script>

<svelte:head>
	<title>Purdue Orbital Ground Station</title>
	<meta name="robots" content="noindex nofollow"/>
</svelte:head>

<Modal show={$modal}>
    <Content/>
</Modal>