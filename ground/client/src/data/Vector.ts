export default class Vector {
    constructor(x: number, y: number, z: number) {
        this._x = x;
        this._y = y;
        this._z = z;
    }

    private _x: number

    public get x(): number {
        return this._x;
    }

    private _y: number;

    public get y(): number {
        return this._y;
    }

    private _z: number;

    public get z(): number {
        return this._z;
    }

}