<script lang="ts">
    import { onDestroy } from "svelte";
    import { modal } from "./stores/ModalStore";

    export let title: string = 'Confirm Action';
    export let body: string = 'Are you sure you want to run this action?';
    export let confirmText: string = 'Yes';
    export let cancelText: string = 'No';
    export let onConfirm: Function = () => {};
    export let onCancel: Function = () => {};

    let didConfirm = false;

    const unsub = modal.subscribe(val => {
        if (!val && !didConfirm) {
            onCancel();
        }
    });

    onDestroy(unsub);

    function confirm() {
        didConfirm = true;
        modal.set(null);
        onConfirm();
    }
    
    function cancel() {
        modal.set(null);
    }
</script>

<div class="root">
    <h2>{title}</h2>
    <p>{body}</p>
    <div class="buttons">
        <button class="button" style="background-color: gray;" on:click={cancel}>{cancelText}</button>
        <button class="button" style="background-color: red;" on:click={confirm}>{confirmText}</button>
    </div>
</div>

<style>
    div {
        align-content: center;
        align-items: center;
        text-align: center;
        justify-content: center;
    }

    .buttons {
        display: flex;
        flex-direction: row;
    }

    .buttons > * {
        margin: 0 5px;
    }
</style>