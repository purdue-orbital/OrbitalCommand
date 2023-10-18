import {writable} from "svelte/store";
import State from "../data/State";

class StateStore {
    public subscribe: Function;
    private _set: Function;
    private _update: Function;

    constructor() {
        let {subscribe, set, update} = writable(this);
        this.subscribe = subscribe;
        this._set = set;
        this._update = update;
    }

    private _launchState: State = State.NotStarted;

    public get launchState(): State {
        return this._launchState;
    }

    private _abortState: State = State.NotStarted;

    public get abortState(): State {
        return this._abortState;
    }

    private _cutState: State = State.NotStarted;

    public get cutState(): State {
        return this._cutState;
    }

    async launch() {
        this._update((that: StateStore) => {
            that._launchState = State.Processing;
            return that;
        });

        const res = await fetch('/launch', {'method': 'POST'});

        if (res.ok) {
            this._update((that: StateStore) => {
                that._launchState = State.Done;
                return that;
            });
        }
    }

    async abort() {
        this._update((that: StateStore) => {
            that._abortState = State.Processing;
            return that;
        });

        const res = await fetch('/abort', {'method': 'POST'});

        if (res.ok) {
            this._update((that: StateStore) => {
                that._abortState = State.Done;
                return that;
            });
        }
    }

    async cut() {
        this._update((that: StateStore) => {
            that._cutState = State.Processing;
            return that;
        });

        const res = await fetch('/cut', {'method': 'POST'});

        if (res.ok) {
            this._update((that: StateStore) => {
                that._cutState = State.Done;
                return that;
            });
        }
    }

    async update() {

    }


}

const stateStore = new StateStore();

export {stateStore, type StateStore}