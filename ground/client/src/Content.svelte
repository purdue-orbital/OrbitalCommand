<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import { scale } from 'svelte/transition';
    import State from './data/State';
    import StateWidgetType from './data/StateWidgetType';
    import Vector from './data/Vector';
    import Map from './Map.svelte';
    import StateWidget from './StateWidget.svelte';
    import { StateStore, stateStore } from './stores/StateStore';
    import {Telemetry, telemetryStore} from './stores/TelemetryStore';

    let interval: NodeJS.Timer = setInterval(() => telemetryStore.update(), 30_000);
    let timeInterval: NodeJS.Timer;
    let start: any = new Date();
    let time: any = new Date();
    
    $: hours = Math.floor(minutes / 60);
    $: minutes = Math.floor(seconds / 60);
    $: seconds = Math.floor(diff / 1000);
    $: millis = diff % 1000;
    $: diff = time - start;

    let disableLaunch: boolean = false;
    let disableAbort: boolean = true;
    let disableCut: boolean = true;

    const zeroPad = (num: number, places: number) => String(num).padStart(places, '0');

    const stateUnsub = stateStore.subscribe((newState: StateStore) => {
        disableAbort = newState.abortState !== State.NotStarted;
        disableLaunch = newState.launchState !== State.NotStarted || disableAbort;
        disableCut = newState.cutState !== State.NotStarted || disableAbort;

        if (newState.abortState === State.Done) {
            clearInterval(timeInterval);
            timeInterval = null;
        }
    });

    let lastTelemUpdate: Date = null;
    let temperature: number = 0;
    let position: Vector = new Vector(0, 0, 0);
    let acceleration: Vector = new Vector(0, 0, 0);

    const telemetryUnsub = telemetryStore.subscribe((newState: Telemetry) => {
        lastTelemUpdate = new Date();
        temperature = newState.temperature;
        position = newState.gps;
        acceleration = newState.accel;
    });

    function clockStart() {
        start = new Date();
        timeInterval = setInterval(() => time = new Date(), 1);
    }

    onMount(() => stateStore.update());

    onDestroy(() => {
        if (timeInterval) clearInterval(timeInterval);
        clearInterval(interval);
        stateUnsub();
        telemetryUnsub();
    });
</script>

<main>
    <div class="header">
        <img src="images/OrbitalLogo.png" class="logo" alt="Purdue Orbital Logo">
        <h1>Ground Station</h1>
    </div>

    <div class="main-page-container">

        <div class="map">
            <Map/>
        </div>
        <div class="statistics">
            Telemetry Data
            {#if lastTelemUpdate}
                <p class="last-updated">Last Updated: {lastTelemUpdate.toLocaleTimeString()}</p>
            {/if}
            <hr>
            <div class="stats-numbers">
                <div>Temp (C): {temperature}</div>
                <div>GPS (x): {position.x}</div>
                <div>GPS (y): {position.y}</div>
                <div>GPS (z): {position.z}</div>
                <div>Acceleration (x): {acceleration.x}</div>
                <div>Acceleration (y): {acceleration.y}</div>
                <div>Acceleration (z): {acceleration.z}</div>
            </div>
        </div>
        
        <StateWidget buttonType={StateWidgetType.LAUNCH} onClick={() => stateStore.launch()} disabled={disableLaunch || !timeInterval}/>
        <StateWidget buttonType={StateWidgetType.ABORT} onClick={() => stateStore.abort()} disabled={disableAbort || !timeInterval}/>
        <StateWidget buttonType={StateWidgetType.CUT} onClick={() => stateStore.cut()} disabled={disableCut || !timeInterval}/>

        <div class="time">
            <h4>Mission Time</h4>
            <h1 class="time-font">
                {hours}:{zeroPad(minutes % 60, 2)}:{zeroPad(seconds % 60, 2)}.{zeroPad(millis, 3)}
            </h1>
            {#if diff === 0}
                <button class="button" on:click={clockStart} style="background-color: green;" transition:scale>Start Clock</button>
            {/if}
        </div>


    </div>
</main>

<style>
    .time-font {
        font-family: monospace;
    }

    .map {
        grid-area: map-grid;
    }
    .statistics {grid-area: statistics-grid;}
    .time {grid-area: time-grid;}

    .main-page-container {
    display: grid;
    grid-template-areas:
        'map-grid map-grid time-grid time-grid time-grid statistics-grid'
        'map-grid map-grid time-grid time-grid time-grid statistics-grid'
        'map-grid map-grid launch-grid abort-grid cut-grid statistics-grid'
        'map-grid map-grid launch-grid abort-grid cut-grid statistics-grid';
    gap: 25px;
    background-color: white;
    padding: 10px;
    }

    :global(.main-page-container > div) {
        background-color: rgb(229, 226, 226);
        text-align: center;
        padding: 20px;
        font-size: 30px;
        border-radius: 10px;
    }

    :global(.main-page-container > div > h4) {
        margin: 10px 0;
    }

    :global(.button) {
        border: none;
        color: white;
        display: inline-block;
        font-size: 15px;
        cursor: pointer;
        padding: 10px;
        border-radius: 10px;
        transition: all 100ms ease;
    }

    :global(.button:hover:enabled) {
        box-shadow: rgba(0, 0, 0, 0.22) 0px 19px 43px;
        transform: translate3d(0px, -1px, 0px);
    }

    :global(.button:disabled) {
        filter: grayscale(50%);
        cursor: not-allowed;
    }

    /* Statistics Area */
    .stats-numbers > div {
        padding: 1.5%;
        text-align: left;
        font-size: 20px;
    }

    :global(.states-text) {
        font-size: 20px;
        padding-top: 10px;
    }

    .logo {
        width: 40%;
        background-color: black;
        padding: 5px;
        border-radius: 3px;
    }

    .header {
        align-content: center;
        text-align: center;
    }

    .last-updated {
        font-size: small;
    }
</style>