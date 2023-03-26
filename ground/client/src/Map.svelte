<script lang="ts">
    import {Map, Marker} from '@beyonk/svelte-mapbox'
    import { onDestroy } from 'svelte';
    import { Telemetry, telemetryStore } from './stores/TelemetryStore';
    let mapComponent;
    let x = 0;
    let y = 0;

    const telemetryUnsub = telemetryStore.subscribe((newState: Telemetry) => {
        if (newState.gps.y !== 0 && newState.gps.x !== 0) {
            mapComponent.flyTo([newState.gps.y, newState.gps.x]);
            x = newState.gps.x;
            y = newState.gps.y;
        }
    });

    onDestroy(telemetryUnsub);

    function onReady() {
        mapComponent.resize();
        mapComponent.setCenter([-86.9141, 40.4273], 22);
    }

    async function getToken() {
        const res = await fetch('/map_token');
        const token = await res.json();

        if (res.ok) {
            return token['token'];
        } else {
            throw new Error(res.statusText);
        }
    }
</script>

{#await getToken()}
    <p>Getting map token...</p>
{:then token}
    <Map accessToken={token} bind:this={mapComponent} on:ready={onReady}>
        <Marker lat={40.4273} lng={-86.9141} label="Purdue"/>
        {#if x !== 0 && y !== 0}
            <Marker lat={y} lng={x} label="Baloon"/>
        {/if}
    </Map>
{:catch error}
    <p style="color: red">Failed to load token: {error.message}</p>
{/await}

<style>
    :global(.mapboxgl-map) {
        max-height:fit-content;
        position: absolute; 
        top: 0; 
        bottom: 0; 
        min-width: 250px;
        border-radius: 10px;
    }
</style>