import {writable} from 'svelte/store';
import Vector from '../data/Vector';

class Telemetry {
    public subscribe: Function;
    private _set: Function;
    private _update: Function;

    constructor() {
        let {subscribe, set, update} = writable(this);
        this.subscribe = subscribe;
        this._set = set;
        this._update = update;
    }

    private _temperature: number = 0;

    public get temperature(): number {
        return this._temperature;
    }

    private _gps: Vector = new Vector(0, 0, 0);

    public get gps(): Vector {
        return this._gps;
    }

    private _accel: Vector = new Vector(0, 0, 0);

    public get accel(): Vector {
        return this._accel;
    }

    async update() {
        await fetch('/update', {'method': 'POST'});

        return new Promise(res => {
            setTimeout(() => {
                fetch('/telemetry').then(r => r.json()).then(vals => {
                    this._update((that: Telemetry) => {
                        if (vals['pos']) that._gps = new Vector(vals['pos'][0], vals['pos'][1], vals['pos'][2]);
                        if (vals['acc']) that._accel = new Vector(vals['acc'][0], vals['acc'][1], vals['acc'][2]);
                        that._temperature = vals['temp'] || 0;
                        return that;
                    })
                    res(null);
                });
            }, 5000)
        });
    }

}

const telemetryStore = new Telemetry();
export {telemetryStore, type Telemetry};